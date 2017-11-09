**Under heavy research and development, please don't use this yet!**

# rsx-layout
[![License: MPL 2.0](https://img.shields.io/badge/License-MPL%202.0-brightgreen.svg)](https://opensource.org/licenses/MPL-2.0)
[![Build Status](https://travis-ci.org/victorporof/rsx-layout.svg?branch=master)](https://travis-ci.org/victorporof/rsx-layout)

A layout tree generated from [RSX DOM](https://github.com/victorporof/rsx-dom) nodes styled with [RSX Stylesheet](https://github.com/victorporof/rsx-stylesheet) rules, using Facebook's [YOGA](https://facebook.github.io/yoga/) as a backend.

## Purpose
Layout trees are data structures representing a collection of renderable boxes with positioning and sizing information.

## How to use
[Documentation](https://victorporof.github.io/rsx-layout)

This crate concerns itself strictly generating a layout tree. If you're just looking to write RSX in your project, take a look at the [RSX compiler plugin](https://github.com/victorporof/rsx_compiler_plugin) instead.

Otherwise, add this to your `Cargo.toml` file:

```toml
[dependencies]
rsx = { git = "https://github.com/victorporof/rsx-compiler-plugin.git" }
rsx-layout = { git = "https://github.com/victorporof/rsx-layout.git" }
```

Then, simply import the library into your code to create an `rsx_layout::LayoutTree` from `rsx_dom::DOMNode` trees styled with `rsx_dom::Stylesheet` rules. See the [RSX compiler plugin](https://github.com/victorporof/rsx-compiler-plugin) for how to create those data structures:

```rust
#![feature(proc_macro)]

extern crate rsx;
extern crate rsx_layout;
...

use rsx::{rsx, css};
use rsx_dom::types::*;
use rsx_stylesheet::types::*;
use rsx_resources::files::types::*;
use rsx_resources::fonts::types::*;
use rsx_resources::types::*;
use rsx_layout::types::{LayoutBoundingClientRect, LayoutReflowDirection};

let mut stylesheet: Stylesheet = css! { ... };
let node: DOMNode = rsx! { ... };

// Load up the necessary resources.
let mut files = FileCache::new().unwrap();
let mut fonts = FontCache::new().unwrap();
let resources = ResourceGroup::new(files, fonts);

// Create a layout tree with a DOM node as root.
node.generate_layout_tree(&resources, None);

// Calculate bounding client rects for subtree starting with a DOM node.
node.reflow_subtree(100.0, 100.0, LayoutReflowDirection::LTR);

// Get a single bounding client rect for a particular DOM node.
let children = node.children_iter().unwrap();
let rect: Option<LayoutBoundingClientRect> = children.next().unwrap().get_bounding_client();
```
