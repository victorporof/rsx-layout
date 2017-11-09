/*
Copyright 2016 Mozilla
Licensed under the Apache License, Version 2.0 (the "License"); you may not use
this file except in compliance with the License. You may obtain a copy of the
License at http://www.apache.org/licenses/LICENSE-2.0
Unless required by applicable law or agreed to in writing, software distributed
under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
CONDITIONS OF ANY KIND, either express or implied. See the License for the
specific language governing permissions and limitations under the License.
*/

use std::convert::TryInto;
use std::fmt;
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Deref, DerefMut};

use rsx_shared::consts::DEFAULT_FONT_SIZE;
use rsx_shared::traits::{
    TClientPosition,
    TClientRect,
    TClientSize,
    TComputedStyles,
    TDOMText,
    TDimensionsInfo,
    TFontCache,
    TGlyphStore,
    TImageCache,
    TLayoutNode,
    TMeasuredImage,
    TResourceGroup,
    TShapedText,
    TStyleDeclarations
};
use rsx_shared::types::KnownElementName;
use yoga;

pub use yoga::Direction as LayoutReflowDirection;

#[derive(Debug, PartialEq, Default, Copy, Clone, Serialize, Deserialize)]
pub struct LayoutBoundingClientRect {
    pub position: LayoutClientPosition,
    pub size: LayoutClientSize
}

impl LayoutBoundingClientRect {
    pub fn new(left: u32, top: u32, width: u32, height: u32) -> Self {
        LayoutBoundingClientRect {
            position: LayoutClientPosition { left, top },
            size: LayoutClientSize { width, height }
        }
    }

    pub fn zero_position(mut self) -> Self {
        self.position = LayoutClientPosition::default();
        self
    }

    pub fn zero_size(mut self) -> Self {
        self.size = LayoutClientSize::default();
        self
    }
}

impl Add<LayoutClientPosition> for LayoutBoundingClientRect {
    type Output = Self;

    fn add(self, rhs: LayoutClientPosition) -> Self::Output {
        let left = self.position.left + rhs.left;
        let top = self.position.top + rhs.top;
        LayoutBoundingClientRect::new(left, top, self.size.width, self.size.height)
    }
}

impl AddAssign<LayoutClientPosition> for LayoutBoundingClientRect {
    fn add_assign(&mut self, rhs: LayoutClientPosition) {
        self.position.left = self.position.left + rhs.left;
        self.position.top = self.position.top + rhs.top;
    }
}

impl TClientRect for LayoutBoundingClientRect {
    type Position = LayoutClientPosition;
    type Size = LayoutClientSize;

    fn position(&self) -> Self::Position {
        self.position
    }

    fn size(&self) -> Self::Size {
        self.size
    }

    fn offset_from_page(&self, (x, y): (u32, u32)) -> (u32, u32) {
        // TODO: Properly handle paddings.
        // See https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/offsetX
        // See https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/offsetY
        (self.position.left + x, self.position.top + y)
    }

    fn client_from_page(&self, (x, y): (u32, u32)) -> (u32, u32) {
        (self.position.left + x, self.position.top + y)
    }

    fn contains_point(&self, (x, y): (u32, u32)) -> bool {
        let p = self.position;
        let s = self.size;
        p.left < x && p.top < y && p.left + s.width > x && p.top + s.height > y
    }
}

#[derive(Debug, PartialEq, Default, Copy, Clone, Serialize, Deserialize)]
pub struct LayoutClientPosition {
    pub left: u32,
    pub top: u32
}

impl Add for LayoutClientPosition {
    type Output = Self;

    fn add(self, rhs: LayoutClientPosition) -> Self::Output {
        let left = self.left + rhs.left;
        let top = self.top + rhs.top;
        LayoutClientPosition { left, top }
    }
}

impl AddAssign for LayoutClientPosition {
    fn add_assign(&mut self, rhs: LayoutClientPosition) {
        self.left = self.left + rhs.left;
        self.top = self.top + rhs.top;
    }
}

impl TClientPosition for LayoutClientPosition {}

#[derive(Debug, PartialEq, Default, Copy, Clone, Serialize, Deserialize)]
pub struct LayoutClientSize {
    pub width: u32,
    pub height: u32
}

impl Add for LayoutClientSize {
    type Output = Self;

    fn add(self, rhs: LayoutClientSize) -> Self::Output {
        let width = self.width + rhs.width;
        let height = self.height + rhs.height;
        LayoutClientSize { width, height }
    }
}

impl AddAssign for LayoutClientSize {
    fn add_assign(&mut self, rhs: LayoutClientSize) {
        self.width = self.width + rhs.width;
        self.height = self.height + rhs.height;
    }
}

impl TClientSize for LayoutClientSize {}

#[derive(Debug, PartialEq)]
pub struct ImageNodeContext {
    width: u32,
    height: u32
}

#[derive(Debug, PartialEq)]
pub struct TextNodeContext {
    width_64: i32,
    height_64: i32
}

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct MeasuredImage<D>(pub(crate) Option<D>);

impl<D> Default for MeasuredImage<D> {
    fn default() -> Self {
        MeasuredImage(None)
    }
}

impl<D> From<D> for MeasuredImage<D> {
    fn from(value: D) -> Self {
        MeasuredImage(Some(value))
    }
}

impl<D> Deref for MeasuredImage<D> {
    type Target = Option<D>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<D> DerefMut for MeasuredImage<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<D> MeasuredImage<D>
where
    D: TDimensionsInfo
{
    #[inline]
    fn should_relayout(&mut self, other: Option<D>) -> bool {
        let should_relayout = self.has_different_layout(&other);
        **self = other;
        should_relayout
    }

    #[inline]
    fn has_different_layout(&self, other: &Option<D>) -> bool {
        match (&self.0, other) {
            (&Some(ref a), &Some(ref b)) => a.width() != b.width() || a.height() != b.height(),
            (&None, &None) => false,
            _ => true
        }
    }

    #[inline]
    pub fn image_key(&self) -> Option<D::ResourceKey> {
        self.as_ref().map(D::resource_key)
    }

    #[inline]
    pub fn width(&self) -> Option<u32> {
        self.as_ref().map(D::width)
    }

    #[inline]
    pub fn height(&self) -> Option<u32> {
        self.as_ref().map(D::height)
    }

    #[inline]
    pub fn aspect_ratio_or(&self, value: f32) -> yoga::FlexStyle {
        let ratio = self.map(|v| v.width() as f32 / v.height() as f32);
        yoga::FlexStyle::AspectRatio(ratio.unwrap_or(value).into())
    }
}

impl<D> TMeasuredImage for MeasuredImage<D>
where
    D: TDimensionsInfo
{
}

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct ShapedText<G>(pub(crate) Option<G>);

impl<G> Default for ShapedText<G> {
    fn default() -> Self {
        ShapedText(None)
    }
}

impl<G> From<G> for ShapedText<G> {
    fn from(value: G) -> Self {
        ShapedText(Some(value))
    }
}

impl<G> Deref for ShapedText<G> {
    type Target = Option<G>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<G> DerefMut for ShapedText<G> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<G> ShapedText<G>
where
    G: TGlyphStore
{
    #[inline]
    fn should_relayout(&mut self, other: Option<G>) -> bool {
        let should_relayout = self.has_different_layout(&other);
        **self = other;
        should_relayout
    }

    #[inline]
    fn has_different_layout(&self, other: &Option<G>) -> bool {
        match (&self.0, other) {
            (&Some(ref a), &Some(ref b)) => a.width_64() != b.width_64() || a.height_64() != b.height_64(),
            (&None, &None) => false,
            _ => true
        }
    }

    #[inline]
    pub fn font_key(&self) -> Option<G::FontKey> {
        self.as_ref().map(G::font_key)
    }

    #[inline]
    pub fn font_instance_key(&self) -> Option<G::FontInstanceKey> {
        self.as_ref().map(G::font_instance_key)
    }

    #[inline]
    pub fn width_f(&self) -> Option<f32> {
        self.as_ref().map(G::width_f)
    }

    #[inline]
    pub fn height_f(&self) -> Option<f32> {
        self.as_ref().map(G::height_f)
    }

    #[inline]
    pub fn width_64(&self) -> Option<i32> {
        self.as_ref().map(G::width_64)
    }

    #[inline]
    pub fn height_64(&self) -> Option<i32> {
        self.as_ref().map(G::height_64)
    }

    #[inline]
    pub fn glyphs(&self) -> &[G::Glyph] {
        self.as_ref().map(G::glyphs).unwrap_or(&[])
    }
}

impl<G> TShapedText for ShapedText<G>
where
    G: TGlyphStore
{
}

pub struct LayoutNode<S, C, R, T>
where
    S: TStyleDeclarations<LayoutStyle = yoga::FlexStyle>,
    C: TComputedStyles<Styles = S>,
    R: TResourceGroup,
    T: TDOMText
{
    tainted: bool,
    layout: yoga::Node,
    computed_client_position: LayoutClientPosition,
    shaped_text: ShapedText<<R::Fonts as TFontCache>::Glyphs>,
    measured_image: MeasuredImage<<R::Images as TImageCache>::Dimensions>,
    phantom: PhantomData<(S, C, R, T)>
}

impl<S, C, R, T> PartialEq for LayoutNode<S, C, R, T>
where
    S: TStyleDeclarations<LayoutStyle = yoga::FlexStyle>,
    C: TComputedStyles<Styles = S>,
    R: TResourceGroup,
    T: TDOMText
{
    fn eq(&self, _: &Self) -> bool {
        // Layout nodes are opaque to the outside world. For equality checks
        // between elements use DOM nodes instead.
        true
    }
}

impl<S, C, R, T> fmt::Debug for LayoutNode<S, C, R, T>
where
    S: TStyleDeclarations<LayoutStyle = yoga::FlexStyle>,
    C: TComputedStyles<Styles = S>,
    R: TResourceGroup,
    T: TDOMText
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "LayoutNode {{ layout: {:?} }}", self.layout)
    }
}

impl<S, C, R, T> TLayoutNode for LayoutNode<S, C, R, T>
where
    S: TStyleDeclarations<LayoutStyle = yoga::FlexStyle>,
    C: TComputedStyles<Styles = S> + 'static,
    R: TResourceGroup + 'static,
    T: TDOMText + 'static
{
    type Styles = S;
    type Resources = R;
    type TextMeasureMetadata = C;
    type ImageMeasureMetadata = ();
    type NormalMeasureMetadata = !;
    type ReflowDirection = LayoutReflowDirection;
    type ClientPosition = LayoutClientPosition;
    type BoundingClientRect = LayoutBoundingClientRect;
    type MeasuredImage = MeasuredImage<<R::Images as TImageCache>::Dimensions>;
    type ShapedText = ShapedText<<R::Fonts as TFontCache>::Glyphs>;

    fn make_initial_layout_node<U>(_: U) -> Self
    where
        U: TryInto<KnownElementName>
    {
        LayoutNode {
            tainted: false,
            layout: yoga::Node::new(),
            computed_client_position: LayoutClientPosition::default(),
            shaped_text: ShapedText::default(),
            measured_image: MeasuredImage::default(),
            phantom: PhantomData
        }
    }

    fn reset_custom_styles<U>(&mut self, _: U)
    where
        U: TryInto<KnownElementName>
    {
        // TODO: actually do something here. Copying styles from a new node
        // seems to have an enormous allocation pressure, which is slow.
        // self.layout.copy_style(&yoga::Node::new());
        // self.tainted = true;
    }

    fn is_tainted(&self) -> bool {
        self.tainted
    }

    fn insert_child(&mut self, child: &mut Self, index: usize) {
        self.layout.insert_child(&mut child.layout, index as u32);
        self.tainted = true;
    }

    fn append_child(&mut self, child: &mut Self) {
        let index = self.layout.child_count();
        self.layout.insert_child(&mut child.layout, index);
        self.tainted = true;
    }

    fn remove_child(&mut self, child: &mut Self) {
        self.layout.remove_child(&mut child.layout);
        self.tainted = true;
    }

    fn apply_styles(&mut self, styles: &Self::Styles) {
        styles.for_each_layout_style(|style| self.layout.apply_style(style));
    }

    fn mark_dirty(&mut self) {
        self.layout.mark_dirty();
    }

    fn measure_self_as_image<U>(&mut self, resources: &Self::Resources, image_src: &U, _: &Self::ImageMeasureMetadata)
    where
        U: TDOMText
    {
        let layout = &mut self.layout;
        let cache = resources.images();
        let new_dimensions = cache.measure_image(&image_src);

        if !self.measured_image.should_relayout(new_dimensions) {
            return;
        }

        layout.set_context(Some(yoga::Context::new(ImageNodeContext {
            width: self.measured_image.width().unwrap_or(0),
            height: self.measured_image.height().unwrap_or(0)
        })));

        layout.set_measure_func(Some(measure_image));
        layout.apply_style(&self.measured_image.aspect_ratio_or(1.0));

        // TODO: Need to mark dirty, but this affects performance quite a lot.
        // Should be smart about it and only mark dirty when actually necesssry.
        // layout.mark_dirty();

        self.tainted = true;
    }

    fn measure_self_as_text<U>(&mut self, resources: &Self::Resources, source_text: &U, metadata: &Self::TextMeasureMetadata)
    where
        U: TDOMText
    {
        let layout = &mut self.layout;
        let cache = resources.fonts();
        let computed_styles = metadata;

        let size = computed_styles
            .font_size()
            .try_into()
            .map(|v| v.point())
            .unwrap_or(DEFAULT_FONT_SIZE);

        let new_glyphs = computed_styles
            .find_font(|name| cache.get_font_with_size(name, size))
            .or_else(|| cache.get_default_font_with_size(size))
            .and_then(|f| cache.shape_text_h(&f, source_text.as_ref()));

        if !self.shaped_text.should_relayout(new_glyphs) {
            return;
        }

        layout.set_context(Some(yoga::Context::new(TextNodeContext {
            width_64: self.shaped_text.width_64().unwrap_or(0),
            height_64: self.shaped_text.height_64().unwrap_or(0)
        })));

        layout.set_measure_func(Some(measure_text));

        // TODO: Need to mark dirty, but this affects performance quite a lot.
        // Should be smart about it and only mark dirty when actually necesssry.
        // layout.mark_dirty();

        self.tainted = true;
    }

    fn measure_self_as_normal(&mut self, _: &Self::Resources, _: &Self::NormalMeasureMetadata) {
        unreachable!()
    }

    fn reflow_subtree(&mut self, width: u32, height: u32, direction: Self::ReflowDirection) {
        let width = width as f32;
        let height = height as f32;
        self.layout.calculate_layout(width, height, direction);
    }

    fn set_computed_client_position(&mut self, computed: Self::ClientPosition) {
        self.computed_client_position = computed;
    }

    fn get_local_bounding_client_rect(&self) -> Self::BoundingClientRect {
        Self::BoundingClientRect::new(
            self.layout.get_layout_left() as u32,
            self.layout.get_layout_top() as u32,
            self.layout.get_layout_width() as u32,
            self.layout.get_layout_height() as u32
        )
    }

    fn get_global_bounding_client_rect(&self) -> Self::BoundingClientRect {
        self.get_local_bounding_client_rect() + self.computed_client_position
    }

    fn get_measured_image(&self) -> &Self::MeasuredImage {
        &self.measured_image
    }

    fn get_shaped_text(&self) -> &Self::ShapedText {
        &self.shaped_text
    }
}

extern "C" fn measure_image(
    node_ref: yoga::YGInternalNodeRef,
    suggested_width: f32,
    node_width_measure_mode: yoga::YGInternalMeasureMode,
    suggested_height: f32,
    node_height_measure_mode: yoga::YGInternalMeasureMode
) -> yoga::YGInternalSize {
    use self::yoga::YGInternalMeasureMode::*;

    let context = yoga::Node::get_context(&node_ref)
        .and_then(|v| v.downcast_ref::<ImageNodeContext>())
        .expect("Invalid context when measuring images.");

    yoga::YGInternalSize {
        width: match (node_width_measure_mode, context.width as f32) {
            (YGMeasureModeExactly, _) => suggested_width,
            (YGMeasureModeAtMost, measured_width) => f32::min(measured_width, suggested_width),
            (YGMeasureModeUndefined, measured_width) => measured_width
        },
        height: match (node_height_measure_mode, context.height as f32) {
            (YGMeasureModeExactly, _) => suggested_height,
            (YGMeasureModeAtMost, measured_height) => f32::min(measured_height, suggested_height),
            (YGMeasureModeUndefined, measured_height) => measured_height
        }
    }
}

extern "C" fn measure_text(
    node_ref: yoga::YGInternalNodeRef,
    suggested_width: f32,
    node_width_measure_mode: yoga::YGInternalMeasureMode,
    suggested_height: f32,
    node_height_measure_mode: yoga::YGInternalMeasureMode
) -> yoga::YGInternalSize {
    use self::yoga::YGInternalMeasureMode::*;

    let context = yoga::Node::get_context(&node_ref)
        .and_then(|v| v.downcast_ref::<TextNodeContext>())
        .expect("Invalid context when measuring text.");

    yoga::YGInternalSize {
        width: match (node_width_measure_mode, context.width_64 as f32 / 64.0) {
            (YGMeasureModeExactly, _) => suggested_width,
            (YGMeasureModeAtMost, measured_width) => f32::min(measured_width, suggested_width),
            (YGMeasureModeUndefined, measured_width) => measured_width
        },
        height: match (node_height_measure_mode, context.height_64 as f32 / 64.0) {
            (YGMeasureModeExactly, _) => suggested_height,
            (YGMeasureModeAtMost, measured_height) => f32::min(measured_height, suggested_height),
            (YGMeasureModeUndefined, measured_height) => measured_height
        }
    }
}
