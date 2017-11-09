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

#![feature(proc_macro)]

extern crate rsx;
#[macro_use]
extern crate rsx_dom;
extern crate rsx_layout;
extern crate rsx_resources;
extern crate rsx_shared;
extern crate rsx_stylesheet;

use rsx::{css, rsx};
use rsx_dom::types::*;
use rsx_dom::types::DOMNode as TTDOMNode;
use rsx_layout::types::*;
use rsx_resources::files::types::*;
use rsx_resources::fonts::types::*;
use rsx_resources::images::types::*;
use rsx_resources::types::*;
use rsx_resources::updates::types::*;
use rsx_shared::traits::*;
use rsx_stylesheet::types::*;

type ImageKeysAPI = DefaultImageKeysAPI;
type FontKeysAPI = DefaultFontKeysAPI;
type DOMNode = TTDOMNode<
    (),
    StyleDeclarations,
    ComputedStyles,
    LayoutNode<StyleDeclarations, ComputedStyles, ResourceGroup<ImageKeysAPI, FontKeysAPI>, DOMText>
>;

#[test]
fn test_reflow_simple() {
    let mut stylesheet = css!("tests/fixtures/test_1.css");

    let mut tree = rsx! {
        <div style={stylesheet.take(".foo")}>
            Hello world!
        </div>
    };

    let expected = fragment! {
        DOMNode::from((
            DOMTagName::from(KnownElementName::Div),
            vec![
                DOMAttribute::from((
                    DOMAttributeName::from(KnownAttributeName::Style),
                    DOMAttributeValue::from(StyleDeclarations(InlineDeclarations::from_vec(vec![
                        StyleDeclaration::Layout(FlexStyle::Position(PositionType::Absolute)),
                        StyleDeclaration::Layout(FlexStyle::Left(StyleUnit::Point(10.0.into()))),
                        StyleDeclaration::Layout(FlexStyle::Top(StyleUnit::Point(20.0.into()))),
                        StyleDeclaration::Layout(FlexStyle::Width(StyleUnit::Point(50.0.into()))),
                        StyleDeclaration::Layout(FlexStyle::Height(StyleUnit::Point(60.0.into()))),
                    ])))
                )),
            ],
            vec![DOMNode::from("Hello world !")]
        ))
    };

    assert_eq!(
        tree.root().traverse_iter().collect::<Vec<_>>(),
        expected.root().traverse_iter().collect::<Vec<_>>()
    );

    let mut files = FileCache::new().unwrap();

    let image_path = "tests/fixtures/Quantum.png";
    assert!(files.add_file(image_path).is_ok());

    let font_path = "tests/fixtures/FreeSans.ttf";
    assert!(files.add_file(font_path).is_ok());

    let image_keys = ImageKeysAPI::new(());
    let mut images = ImageCache::new(image_keys).unwrap();

    let font_keys = FontKeysAPI::new(());
    let mut fonts = FontCache::new(font_keys).unwrap();

    let image_id = ImageId::new("logo");
    let image_bytes = files.get_file(image_path).unwrap();
    images.add_raw(image_id, image_bytes).unwrap();

    let font_id = FontId::new("FreeSans");
    let font_bytes = files.get_file(font_path).unwrap();
    fonts.add_raw(font_id, font_bytes, 0).unwrap();

    let resources = ResourceGroup::new(files, images, fonts);
    tree.generate_layout_tree(&resources);
    tree.reflow_subtree(100, 100, LayoutReflowDirection::LTR);

    let mut descendants = tree.root().descendants_iter();

    assert_eq!(
        descendants.next().unwrap().get_local_bounding_client_rect(),
        LayoutBoundingClientRect::new(0, 0, 100, 100)
    );

    assert_eq!(
        descendants.next().unwrap().get_local_bounding_client_rect(),
        LayoutBoundingClientRect::new(10, 20, 50, 60)
    );

    assert_eq!(
        descendants.next().unwrap().get_local_bounding_client_rect(),
        LayoutBoundingClientRect::new(0, 0, 50, 60)
    );
}

#[test]
fn test_reflow_example_1() {
    let mut stylesheet = css!("tests/fixtures/test_2.css");

    let mut tree = rsx! {
        <root style={stylesheet.take(".root")}>
            <image src={"logo"} style={stylesheet.take(".image")} />
            <text style={stylesheet.take(".text")} />
        </root>
    };

    let expected = fragment! {
        DOMNode::from((
            DOMTagName::from("root"),
            vec![
                DOMAttribute::from((
                    DOMAttributeName::from(KnownAttributeName::Style),
                    DOMAttributeValue::from(StyleDeclarations(InlineDeclarations::from_vec(vec![
                        StyleDeclaration::Layout(FlexStyle::Width(StyleUnit::Point(500.0.into()))),
                        StyleDeclaration::Layout(FlexStyle::Height(StyleUnit::Point(120.0.into()))),
                        StyleDeclaration::Layout(FlexStyle::FlexDirection(FlexDirection::Row)),
                        StyleDeclaration::Layout(FlexStyle::PaddingTop(StyleUnit::Point(20.0.into()))),
                        StyleDeclaration::Layout(FlexStyle::PaddingRight(StyleUnit::Point(20.0.into()))),
                        StyleDeclaration::Layout(FlexStyle::PaddingBottom(StyleUnit::Point(20.0.into()))),
                        StyleDeclaration::Layout(FlexStyle::PaddingLeft(StyleUnit::Point(20.0.into()))),
                    ])))
                )),
            ],
            vec![
                DOMNode::from((
                    DOMTagName::from(KnownElementName::Image),
                    vec![
                        DOMAttribute::from((
                            DOMAttributeName::from(KnownAttributeName::Src),
                            DOMAttributeValue::from("logo")
                        )),
                        DOMAttribute::from((
                            DOMAttributeName::from(KnownAttributeName::Style),
                            DOMAttributeValue::from(StyleDeclarations(InlineDeclarations::from_vec(vec![
                                StyleDeclaration::Layout(FlexStyle::Width(StyleUnit::Point(80.0.into()))),
                                StyleDeclaration::Layout(FlexStyle::MarginRight(StyleUnit::Point(20.0.into()))),
                            ])))
                        )),
                    ]
                )),
                DOMNode::from((
                    DOMTagName::from(KnownElementName::Text),
                    vec![
                        DOMAttribute::from((
                            DOMAttributeName::from(KnownAttributeName::Style),
                            DOMAttributeValue::from(StyleDeclarations(InlineDeclarations::from_vec(vec![
                                StyleDeclaration::Layout(FlexStyle::Height(StyleUnit::Point(25.0.into()))),
                                StyleDeclaration::Layout(FlexStyle::AlignSelf(Align::Center)),
                                StyleDeclaration::Layout(FlexStyle::FlexGrow(1.0.into())),
                            ])))
                        )),
                    ]
                )),
            ]
        ))
    };

    assert_eq!(
        tree.root().traverse_iter().collect::<Vec<_>>(),
        expected.root().traverse_iter().collect::<Vec<_>>()
    );

    let mut files = FileCache::new().unwrap();

    let image_path = "tests/fixtures/Quantum.png";
    assert!(files.add_file(image_path).is_ok());

    let font_path = "tests/fixtures/FreeSans.ttf";
    assert!(files.add_file(font_path).is_ok());

    let image_keys = ImageKeysAPI::new(());
    let mut images = ImageCache::new(image_keys).unwrap();

    let font_keys = FontKeysAPI::new(());
    let mut fonts = FontCache::new(font_keys).unwrap();

    let image_id = ImageId::new("logo");
    let image_bytes = files.get_file(image_path).unwrap();
    images.add_raw(image_id, image_bytes).unwrap();

    let font_id = FontId::new("FreeSans");
    let font_bytes = files.get_file(font_path).unwrap();
    fonts.add_raw(font_id, font_bytes, 0).unwrap();

    let resources = ResourceGroup::new(files, images, fonts);
    tree.generate_layout_tree(&resources);
    tree.reflow_subtree(1000, 1000, LayoutReflowDirection::LTR);

    let mut descendants = tree.root().descendants_iter();

    assert_eq!(
        descendants.next().unwrap().get_local_bounding_client_rect(),
        LayoutBoundingClientRect::new(0, 0, 1000, 1000)
    );

    assert_eq!(
        descendants.next().unwrap().get_local_bounding_client_rect(),
        LayoutBoundingClientRect::new(0, 0, 500, 120)
    );

    assert_eq!(
        descendants.next().unwrap().get_local_bounding_client_rect(),
        LayoutBoundingClientRect::new(20, 20, 80, 83)
    );

    assert_eq!(
        descendants.next().unwrap().get_local_bounding_client_rect(),
        LayoutBoundingClientRect::new(120, 48, 360, 25)
    );
}

#[test]
fn test_reflow_example_2() {
    let mut stylesheet = css!("tests/fixtures/test_3.css");

    let mut tree = rsx! {
        <root style={stylesheet.take(".root")}>
            <image src={"logo"} style={stylesheet.take(".image")} />
            <text style={stylesheet.take(".text")} />
        </root>
    };

    let expected = fragment! {
        DOMNode::from((
            DOMTagName::from("root"),
            vec![
                DOMAttribute::from((
                    DOMAttributeName::from(KnownAttributeName::Style),
                    DOMAttributeValue::from(StyleDeclarations(InlineDeclarations::from_vec(vec![
                        StyleDeclaration::Theme(ThemeStyle::BackgroundColor(Color {
                            red: 255,
                            green: 0,
                            blue: 0,
                            alpha: 255
                        })),
                        StyleDeclaration::Layout(FlexStyle::Width(StyleUnit::Point(500.0.into()))),
                        StyleDeclaration::Layout(FlexStyle::Height(StyleUnit::Point(120.0.into()))),
                        StyleDeclaration::Layout(FlexStyle::FlexDirection(FlexDirection::Row)),
                        StyleDeclaration::Layout(FlexStyle::PaddingTop(StyleUnit::Point(20.0.into()))),
                        StyleDeclaration::Layout(FlexStyle::PaddingRight(StyleUnit::Point(20.0.into()))),
                        StyleDeclaration::Layout(FlexStyle::PaddingBottom(StyleUnit::Point(20.0.into()))),
                        StyleDeclaration::Layout(FlexStyle::PaddingLeft(StyleUnit::Point(20.0.into()))),
                    ])))
                )),
            ],
            vec![
                DOMNode::from((
                    DOMTagName::from(KnownElementName::Image),
                    vec![
                        DOMAttribute::from((
                            DOMAttributeName::from(KnownAttributeName::Src),
                            DOMAttributeValue::from("logo")
                        )),
                        DOMAttribute::from((
                            DOMAttributeName::from(KnownAttributeName::Style),
                            DOMAttributeValue::from(StyleDeclarations(InlineDeclarations::from_vec(vec![
                                StyleDeclaration::Theme(ThemeStyle::BackgroundColor(Color {
                                    red: 0,
                                    green: 128,
                                    blue: 0,
                                    alpha: 255
                                })),
                                StyleDeclaration::Theme(ThemeStyle::Opacity(50)),
                                StyleDeclaration::Layout(FlexStyle::Width(StyleUnit::Point(80.0.into()))),
                                StyleDeclaration::Layout(FlexStyle::MarginRight(StyleUnit::Point(20.0.into()))),
                            ])))
                        )),
                    ]
                )),
                DOMNode::from((
                    DOMTagName::from(KnownElementName::Text),
                    vec![
                        DOMAttribute::from((
                            DOMAttributeName::from(KnownAttributeName::Style),
                            DOMAttributeValue::from(StyleDeclarations(InlineDeclarations::from_vec(vec![
                                StyleDeclaration::Theme(ThemeStyle::BackgroundColor(Color {
                                    red: 0,
                                    green: 0,
                                    blue: 255,
                                    alpha: 255
                                })),
                                StyleDeclaration::Theme(ThemeStyle::Color(Color {
                                    red: 255,
                                    green: 255,
                                    blue: 0,
                                    alpha: 255
                                })),
                                StyleDeclaration::Layout(FlexStyle::Height(StyleUnit::Point(25.0.into()))),
                                StyleDeclaration::Layout(FlexStyle::AlignSelf(Align::Center)),
                                StyleDeclaration::Layout(FlexStyle::FlexGrow(1.0.into())),
                            ])))
                        )),
                    ]
                )),
            ]
        ))
    };

    assert_eq!(
        tree.root().traverse_iter().collect::<Vec<_>>(),
        expected.root().traverse_iter().collect::<Vec<_>>()
    );

    let mut files = FileCache::new().unwrap();

    let image_path = "tests/fixtures/Quantum.png";
    assert!(files.add_file(image_path).is_ok());

    let font_path = "tests/fixtures/FreeSans.ttf";
    assert!(files.add_file(font_path).is_ok());

    let image_keys = ImageKeysAPI::new(());
    let mut images = ImageCache::new(image_keys).unwrap();

    let font_keys = FontKeysAPI::new(());
    let mut fonts = FontCache::new(font_keys).unwrap();

    let image_id = ImageId::new("logo");
    let image_bytes = files.get_file(image_path).unwrap();
    images.add_raw(image_id, image_bytes).unwrap();

    let font_id = FontId::new("FreeSans");
    let font_bytes = files.get_file(font_path).unwrap();
    fonts.add_raw(font_id, font_bytes, 0).unwrap();

    let resources = ResourceGroup::new(files, images, fonts);
    tree.generate_layout_tree(&resources);
    tree.reflow_subtree(1000, 1000, LayoutReflowDirection::LTR);

    let mut descendants = tree.root().descendants_iter();

    assert_eq!(
        descendants.next().unwrap().get_local_bounding_client_rect(),
        LayoutBoundingClientRect::new(0, 0, 1000, 1000)
    );

    assert_eq!(
        descendants.next().unwrap().get_local_bounding_client_rect(),
        LayoutBoundingClientRect::new(0, 0, 500, 120)
    );

    assert_eq!(
        descendants.next().unwrap().get_local_bounding_client_rect(),
        LayoutBoundingClientRect::new(20, 20, 80, 83)
    );

    assert_eq!(
        descendants.next().unwrap().get_local_bounding_client_rect(),
        LayoutBoundingClientRect::new(120, 48, 360, 25)
    );
}
