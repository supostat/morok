#[derive(Clone, Copy, Debug)]
pub struct Classes {
    pub button: ::morok_style::Class,
    pub primary: ::morok_style::Class,
    pub toolbar: ::morok_style::Class,
    pub panel: ::morok_style::Class,
    pub badge: ::morok_style::Class,
}
pub const CLASSES: Classes = Classes {
    button: ::morok_style::Class::new(0),
    primary: ::morok_style::Class::new(1),
    toolbar: ::morok_style::Class::new(2),
    panel: ::morok_style::Class::new(3),
    badge: ::morok_style::Class::new(4),
};
pub const STYLE_SHEET: ::morok_style::StyleSheet = ::morok_style::StyleSheet {
    rules: ::std::borrow::Cow::Borrowed(
        &[
            ::morok_style::StyleRule {
                selector: ::morok_style::Selector {
                    subject: ::morok_style::CompoundSelector {
                        classes: ::std::borrow::Cow::Borrowed(
                            &[::morok_style::Class::new(0)],
                        ),
                        states: ::morok_style::States::NONE,
                    },
                    parent: ::core::option::Option::None,
                },
                declarations: ::std::borrow::Cow::Borrowed(
                    &[
                        ::morok_style::Declaration::Display(
                            ::morok_style::Display::Flex,
                        ),
                        ::morok_style::Declaration::AlignItems(
                            ::morok_style::ItemAlignment::Center,
                        ),
                        ::morok_style::Declaration::JustifyContent(
                            ::morok_style::ContentAlignment::Center,
                        ),
                        ::morok_style::Declaration::RowGap(::morok_style::LengthPercentage {
                            pixels: ::morok_style::Pixels(0.0),
                            fraction: ::core::option::Option::None,
                            tokens: ::std::borrow::Cow::Borrowed(
                                &[
                                    ::morok_style::TokenTerm {
                                        token: ::morok_style::Token::new(1),
                                        factor: 1.0,
                                    },
                                ],
                            ),
                        }),
                        ::morok_style::Declaration::ColumnGap(::morok_style::LengthPercentage {
                            pixels: ::morok_style::Pixels(0.0),
                            fraction: ::core::option::Option::None,
                            tokens: ::std::borrow::Cow::Borrowed(
                                &[
                                    ::morok_style::TokenTerm {
                                        token: ::morok_style::Token::new(1),
                                        factor: 1.0,
                                    },
                                ],
                            ),
                        }),
                        ::morok_style::Declaration::Height(
                            ::morok_style::LengthPercentageAuto::LengthPercentage(::morok_style::LengthPercentage {
                                pixels: ::morok_style::Pixels(28.0),
                                fraction: ::core::option::Option::None,
                                tokens: ::std::borrow::Cow::Borrowed(&[]),
                            }),
                        ),
                        ::morok_style::Declaration::Padding(
                            ::morok_style::Side::Top,
                            ::morok_style::LengthPercentage {
                                pixels: ::morok_style::Pixels(0.0),
                                fraction: ::core::option::Option::None,
                                tokens: ::std::borrow::Cow::Borrowed(&[]),
                            },
                        ),
                        ::morok_style::Declaration::Padding(
                            ::morok_style::Side::Right,
                            ::morok_style::LengthPercentage {
                                pixels: ::morok_style::Pixels(0.0),
                                fraction: ::core::option::Option::None,
                                tokens: ::std::borrow::Cow::Borrowed(
                                    &[
                                        ::morok_style::TokenTerm {
                                            token: ::morok_style::Token::new(2),
                                            factor: 1.0,
                                        },
                                    ],
                                ),
                            },
                        ),
                        ::morok_style::Declaration::Padding(
                            ::morok_style::Side::Bottom,
                            ::morok_style::LengthPercentage {
                                pixels: ::morok_style::Pixels(0.0),
                                fraction: ::core::option::Option::None,
                                tokens: ::std::borrow::Cow::Borrowed(&[]),
                            },
                        ),
                        ::morok_style::Declaration::Padding(
                            ::morok_style::Side::Left,
                            ::morok_style::LengthPercentage {
                                pixels: ::morok_style::Pixels(0.0),
                                fraction: ::core::option::Option::None,
                                tokens: ::std::borrow::Cow::Borrowed(
                                    &[
                                        ::morok_style::TokenTerm {
                                            token: ::morok_style::Token::new(2),
                                            factor: 1.0,
                                        },
                                    ],
                                ),
                            },
                        ),
                        ::morok_style::Declaration::BorderWidth(
                            ::morok_style::Side::Top,
                            ::morok_style::Length {
                                pixels: ::morok_style::Pixels(1.0),
                                tokens: ::std::borrow::Cow::Borrowed(&[]),
                            },
                        ),
                        ::morok_style::Declaration::BorderStyle(
                            ::morok_style::Side::Top,
                            ::morok_style::LineStyle::Solid,
                        ),
                        ::morok_style::Declaration::BorderColor(
                            ::morok_style::Side::Top,
                            ::morok_style::ColorValue::Token(
                                ::morok_style::Token::new(4),
                            ),
                        ),
                        ::morok_style::Declaration::BorderWidth(
                            ::morok_style::Side::Right,
                            ::morok_style::Length {
                                pixels: ::morok_style::Pixels(1.0),
                                tokens: ::std::borrow::Cow::Borrowed(&[]),
                            },
                        ),
                        ::morok_style::Declaration::BorderStyle(
                            ::morok_style::Side::Right,
                            ::morok_style::LineStyle::Solid,
                        ),
                        ::morok_style::Declaration::BorderColor(
                            ::morok_style::Side::Right,
                            ::morok_style::ColorValue::Token(
                                ::morok_style::Token::new(4),
                            ),
                        ),
                        ::morok_style::Declaration::BorderWidth(
                            ::morok_style::Side::Bottom,
                            ::morok_style::Length {
                                pixels: ::morok_style::Pixels(1.0),
                                tokens: ::std::borrow::Cow::Borrowed(&[]),
                            },
                        ),
                        ::morok_style::Declaration::BorderStyle(
                            ::morok_style::Side::Bottom,
                            ::morok_style::LineStyle::Solid,
                        ),
                        ::morok_style::Declaration::BorderColor(
                            ::morok_style::Side::Bottom,
                            ::morok_style::ColorValue::Token(
                                ::morok_style::Token::new(4),
                            ),
                        ),
                        ::morok_style::Declaration::BorderWidth(
                            ::morok_style::Side::Left,
                            ::morok_style::Length {
                                pixels: ::morok_style::Pixels(1.0),
                                tokens: ::std::borrow::Cow::Borrowed(&[]),
                            },
                        ),
                        ::morok_style::Declaration::BorderStyle(
                            ::morok_style::Side::Left,
                            ::morok_style::LineStyle::Solid,
                        ),
                        ::morok_style::Declaration::BorderColor(
                            ::morok_style::Side::Left,
                            ::morok_style::ColorValue::Token(
                                ::morok_style::Token::new(4),
                            ),
                        ),
                        ::morok_style::Declaration::BorderRadius(
                            ::morok_style::Corner::TopLeft,
                            ::morok_style::Length {
                                pixels: ::morok_style::Pixels(0.0),
                                tokens: ::std::borrow::Cow::Borrowed(
                                    &[
                                        ::morok_style::TokenTerm {
                                            token: ::morok_style::Token::new(9),
                                            factor: 1.0,
                                        },
                                    ],
                                ),
                            },
                        ),
                        ::morok_style::Declaration::BorderRadius(
                            ::morok_style::Corner::TopRight,
                            ::morok_style::Length {
                                pixels: ::morok_style::Pixels(0.0),
                                tokens: ::std::borrow::Cow::Borrowed(
                                    &[
                                        ::morok_style::TokenTerm {
                                            token: ::morok_style::Token::new(9),
                                            factor: 1.0,
                                        },
                                    ],
                                ),
                            },
                        ),
                        ::morok_style::Declaration::BorderRadius(
                            ::morok_style::Corner::BottomRight,
                            ::morok_style::Length {
                                pixels: ::morok_style::Pixels(0.0),
                                tokens: ::std::borrow::Cow::Borrowed(
                                    &[
                                        ::morok_style::TokenTerm {
                                            token: ::morok_style::Token::new(9),
                                            factor: 1.0,
                                        },
                                    ],
                                ),
                            },
                        ),
                        ::morok_style::Declaration::BorderRadius(
                            ::morok_style::Corner::BottomLeft,
                            ::morok_style::Length {
                                pixels: ::morok_style::Pixels(0.0),
                                tokens: ::std::borrow::Cow::Borrowed(
                                    &[
                                        ::morok_style::TokenTerm {
                                            token: ::morok_style::Token::new(9),
                                            factor: 1.0,
                                        },
                                    ],
                                ),
                            },
                        ),
                        ::morok_style::Declaration::BackgroundColor(
                            ::morok_style::ColorValue::Token(
                                ::morok_style::Token::new(1),
                            ),
                        ),
                        ::morok_style::Declaration::BackgroundImage(
                            ::morok_style::BackgroundImage::None,
                        ),
                        ::morok_style::Declaration::Color(
                            ::morok_style::ColorValue::Token(
                                ::morok_style::Token::new(6),
                            ),
                        ),
                        ::morok_style::Declaration::BoxShadow(
                            ::morok_style::BoxShadow::Token(::morok_style::Token::new(0)),
                        ),
                    ],
                ),
            },
            ::morok_style::StyleRule {
                selector: ::morok_style::Selector {
                    subject: ::morok_style::CompoundSelector {
                        classes: ::std::borrow::Cow::Borrowed(
                            &[::morok_style::Class::new(3)],
                        ),
                        states: ::morok_style::States::NONE,
                    },
                    parent: ::core::option::Option::None,
                },
                declarations: ::std::borrow::Cow::Borrowed(
                    &[
                        ::morok_style::Declaration::Display(
                            ::morok_style::Display::Grid,
                        ),
                        ::morok_style::Declaration::GridTemplateColumns(
                            ::std::borrow::Cow::Borrowed(
                                &[
                                    ::morok_style::TrackListItem::Track(
                                        ::morok_style::TrackSize::Breadth(
                                            ::morok_style::TrackBreadth::LengthPercentage(::morok_style::LengthPercentage {
                                                pixels: ::morok_style::Pixels(190.0),
                                                fraction: ::core::option::Option::None,
                                                tokens: ::std::borrow::Cow::Borrowed(&[]),
                                            }),
                                        ),
                                    ),
                                    ::morok_style::TrackListItem::Track(::morok_style::TrackSize::MinMax {
                                        min: ::morok_style::TrackBreadth::LengthPercentage(::morok_style::LengthPercentage {
                                            pixels: ::morok_style::Pixels(0.0),
                                            fraction: ::core::option::Option::None,
                                            tokens: ::std::borrow::Cow::Borrowed(&[]),
                                        }),
                                        max: ::morok_style::TrackBreadth::Flex(1.0),
                                    }),
                                    ::morok_style::TrackListItem::Track(
                                        ::morok_style::TrackSize::FitContent(::morok_style::LengthPercentage {
                                            pixels: ::morok_style::Pixels(250.0),
                                            fraction: ::core::option::Option::None,
                                            tokens: ::std::borrow::Cow::Borrowed(&[]),
                                        }),
                                    ),
                                ],
                            ),
                        ),
                        ::morok_style::Declaration::GridTemplateRows(
                            ::std::borrow::Cow::Borrowed(
                                &[
                                    ::morok_style::TrackListItem::Repeat {
                                        count: ::morok_style::RepeatCount::Times(2),
                                        tracks: ::std::borrow::Cow::Borrowed(
                                            &[
                                                ::morok_style::TrackSize::Breadth(
                                                    ::morok_style::TrackBreadth::Auto,
                                                ),
                                            ],
                                        ),
                                    },
                                    ::morok_style::TrackListItem::Repeat {
                                        count: ::morok_style::RepeatCount::AutoFill,
                                        tracks: ::std::borrow::Cow::Borrowed(
                                            &[
                                                ::morok_style::TrackSize::MinMax {
                                                    min: ::morok_style::TrackBreadth::LengthPercentage(::morok_style::LengthPercentage {
                                                        pixels: ::morok_style::Pixels(24.0),
                                                        fraction: ::core::option::Option::None,
                                                        tokens: ::std::borrow::Cow::Borrowed(&[]),
                                                    }),
                                                    max: ::morok_style::TrackBreadth::MaxContent,
                                                },
                                            ],
                                        ),
                                    },
                                ],
                            ),
                        ),
                        ::morok_style::Declaration::GridAutoRows(
                            ::std::borrow::Cow::Borrowed(
                                &[
                                    ::morok_style::TrackSize::Breadth(
                                        ::morok_style::TrackBreadth::MinContent,
                                    ),
                                ],
                            ),
                        ),
                        ::morok_style::Declaration::GridAutoColumns(
                            ::std::borrow::Cow::Borrowed(
                                &[
                                    ::morok_style::TrackSize::Breadth(
                                        ::morok_style::TrackBreadth::Flex(1.0),
                                    ),
                                    ::morok_style::TrackSize::Breadth(
                                        ::morok_style::TrackBreadth::Flex(2.0),
                                    ),
                                ],
                            ),
                        ),
                        ::morok_style::Declaration::GridAutoFlow(
                            ::morok_style::GridAutoFlow::ColumnDense,
                        ),
                        ::morok_style::Declaration::JustifyItems(
                            ::morok_style::ItemAlignment::Stretch,
                        ),
                        ::morok_style::Declaration::AlignContent(
                            ::morok_style::ContentAlignment::SpaceBetween,
                        ),
                        ::morok_style::Declaration::Width(
                            ::morok_style::LengthPercentageAuto::LengthPercentage(::morok_style::LengthPercentage {
                                pixels: ::morok_style::Pixels(0.0),
                                fraction: ::core::option::Option::Some(1.0),
                                tokens: ::std::borrow::Cow::Borrowed(
                                    &[
                                        ::morok_style::TokenTerm {
                                            token: ::morok_style::Token::new(3),
                                            factor: -2.0,
                                        },
                                    ],
                                ),
                            }),
                        ),
                        ::morok_style::Declaration::MinWidth(
                            ::morok_style::LengthPercentageAuto::LengthPercentage(::morok_style::LengthPercentage {
                                pixels: ::morok_style::Pixels(0.0),
                                fraction: ::core::option::Option::None,
                                tokens: ::std::borrow::Cow::Borrowed(&[]),
                            }),
                        ),
                        ::morok_style::Declaration::MaxHeight(
                            ::morok_style::LengthPercentageAuto::Auto,
                        ),
                        ::morok_style::Declaration::OverflowX(
                            ::morok_style::Overflow::Hidden,
                        ),
                        ::morok_style::Declaration::OverflowY(
                            ::morok_style::Overflow::Visible,
                        ),
                        ::morok_style::Declaration::BackgroundImage(
                            ::morok_style::BackgroundImage::Gradient(
                                ::morok_style::Gradient::Linear(::morok_style::LinearGradient {
                                    direction: ::morok_style::GradientDirection::ToCorner(
                                        ::morok_style::Corner::BottomRight,
                                    ),
                                    stops: ::std::borrow::Cow::Borrowed(
                                        &[
                                            ::morok_style::ColorStop {
                                                color: ::morok_style::ColorValue::Token(
                                                    ::morok_style::Token::new(1),
                                                ),
                                                position: ::core::option::Option::None,
                                            },
                                            ::morok_style::ColorStop {
                                                color: ::morok_style::ColorValue::Token(
                                                    ::morok_style::Token::new(2),
                                                ),
                                                position: ::core::option::Option::Some(::morok_style::LengthPercentage {
                                                    pixels: ::morok_style::Pixels(0.0),
                                                    fraction: ::core::option::Option::Some(0.6),
                                                    tokens: ::std::borrow::Cow::Borrowed(&[]),
                                                }),
                                            },
                                            ::morok_style::ColorStop {
                                                color: ::morok_style::ColorValue::Transparent,
                                                position: ::core::option::Option::None,
                                            },
                                        ],
                                    ),
                                }),
                            ),
                        ),
                        ::morok_style::Declaration::BorderRadius(
                            ::morok_style::Corner::TopLeft,
                            ::morok_style::Length {
                                pixels: ::morok_style::Pixels(0.0),
                                tokens: ::std::borrow::Cow::Borrowed(
                                    &[
                                        ::morok_style::TokenTerm {
                                            token: ::morok_style::Token::new(11),
                                            factor: 1.0,
                                        },
                                    ],
                                ),
                            },
                        ),
                        ::morok_style::Declaration::BorderRadius(
                            ::morok_style::Corner::TopRight,
                            ::morok_style::Length {
                                pixels: ::morok_style::Pixels(0.0),
                                tokens: ::std::borrow::Cow::Borrowed(
                                    &[
                                        ::morok_style::TokenTerm {
                                            token: ::morok_style::Token::new(11),
                                            factor: 1.0,
                                        },
                                    ],
                                ),
                            },
                        ),
                        ::morok_style::Declaration::BorderRadius(
                            ::morok_style::Corner::BottomRight,
                            ::morok_style::Length {
                                pixels: ::morok_style::Pixels(0.0),
                                tokens: ::std::borrow::Cow::Borrowed(&[]),
                            },
                        ),
                        ::morok_style::Declaration::BorderRadius(
                            ::morok_style::Corner::BottomLeft,
                            ::morok_style::Length {
                                pixels: ::morok_style::Pixels(0.0),
                                tokens: ::std::borrow::Cow::Borrowed(&[]),
                            },
                        ),
                    ],
                ),
            },
            ::morok_style::StyleRule {
                selector: ::morok_style::Selector {
                    subject: ::morok_style::CompoundSelector {
                        classes: ::std::borrow::Cow::Borrowed(
                            &[::morok_style::Class::new(0)],
                        ),
                        states: ::morok_style::States::HOVER,
                    },
                    parent: ::core::option::Option::None,
                },
                declarations: ::std::borrow::Cow::Borrowed(
                    &[
                        ::morok_style::Declaration::BackgroundColor(
                            ::morok_style::ColorValue::Token(
                                ::morok_style::Token::new(2),
                            ),
                        ),
                    ],
                ),
            },
            ::morok_style::StyleRule {
                selector: ::morok_style::Selector {
                    subject: ::morok_style::CompoundSelector {
                        classes: ::std::borrow::Cow::Borrowed(
                            &[::morok_style::Class::new(0)],
                        ),
                        states: ::morok_style::States::FOCUS_VISIBLE,
                    },
                    parent: ::core::option::Option::None,
                },
                declarations: ::std::borrow::Cow::Borrowed(
                    &[
                        ::morok_style::Declaration::OutlineWidth(::morok_style::Length {
                            pixels: ::morok_style::Pixels(2.0),
                            tokens: ::std::borrow::Cow::Borrowed(&[]),
                        }),
                        ::morok_style::Declaration::OutlineStyle(
                            ::morok_style::LineStyle::Solid,
                        ),
                        ::morok_style::Declaration::OutlineColor(
                            ::morok_style::ColorValue::Token(
                                ::morok_style::Token::new(15),
                            ),
                        ),
                        ::morok_style::Declaration::OutlineOffset(::morok_style::Length {
                            pixels: ::morok_style::Pixels(2.0),
                            tokens: ::std::borrow::Cow::Borrowed(&[]),
                        }),
                    ],
                ),
            },
            ::morok_style::StyleRule {
                selector: ::morok_style::Selector {
                    subject: ::morok_style::CompoundSelector {
                        classes: ::std::borrow::Cow::Borrowed(
                            &[::morok_style::Class::new(0), ::morok_style::Class::new(1)],
                        ),
                        states: ::morok_style::States::NONE,
                    },
                    parent: ::core::option::Option::None,
                },
                declarations: ::std::borrow::Cow::Borrowed(
                    &[
                        ::morok_style::Declaration::BackgroundColor(
                            ::morok_style::ColorValue::Token(
                                ::morok_style::Token::new(11),
                            ),
                        ),
                        ::morok_style::Declaration::BackgroundImage(
                            ::morok_style::BackgroundImage::None,
                        ),
                        ::morok_style::Declaration::Color(
                            ::morok_style::ColorValue::Token(
                                ::morok_style::Token::new(9),
                            ),
                        ),
                        ::morok_style::Declaration::BorderColor(
                            ::morok_style::Side::Top,
                            ::morok_style::ColorValue::Transparent,
                        ),
                        ::morok_style::Declaration::BorderColor(
                            ::morok_style::Side::Right,
                            ::morok_style::ColorValue::Transparent,
                        ),
                        ::morok_style::Declaration::BorderColor(
                            ::morok_style::Side::Bottom,
                            ::morok_style::ColorValue::Transparent,
                        ),
                        ::morok_style::Declaration::BorderColor(
                            ::morok_style::Side::Left,
                            ::morok_style::ColorValue::Transparent,
                        ),
                    ],
                ),
            },
            ::morok_style::StyleRule {
                selector: ::morok_style::Selector {
                    subject: ::morok_style::CompoundSelector {
                        classes: ::std::borrow::Cow::Borrowed(
                            &[::morok_style::Class::new(0)],
                        ),
                        states: ::morok_style::States::DISABLED,
                    },
                    parent: ::core::option::Option::None,
                },
                declarations: ::std::borrow::Cow::Borrowed(
                    &[
                        ::morok_style::Declaration::BoxShadow(
                            ::morok_style::BoxShadow::None,
                        ),
                    ],
                ),
            },
            ::morok_style::StyleRule {
                selector: ::morok_style::Selector {
                    subject: ::morok_style::CompoundSelector {
                        classes: ::std::borrow::Cow::Borrowed(
                            &[::morok_style::Class::new(0)],
                        ),
                        states: ::morok_style::States::NONE,
                    },
                    parent: ::core::option::Option::Some(::morok_style::CompoundSelector {
                        classes: ::std::borrow::Cow::Borrowed(
                            &[::morok_style::Class::new(2)],
                        ),
                        states: ::morok_style::States::NONE,
                    }),
                },
                declarations: ::std::borrow::Cow::Borrowed(
                    &[
                        ::morok_style::Declaration::FlexGrow(0.0),
                        ::morok_style::Declaration::FlexShrink(0.0),
                        ::morok_style::Declaration::FlexBasis(
                            ::morok_style::LengthPercentageAuto::Auto,
                        ),
                        ::morok_style::Declaration::Margin(
                            ::morok_style::Side::Top,
                            ::morok_style::LengthPercentageAuto::LengthPercentage(::morok_style::LengthPercentage {
                                pixels: ::morok_style::Pixels(0.0),
                                fraction: ::core::option::Option::None,
                                tokens: ::std::borrow::Cow::Borrowed(&[]),
                            }),
                        ),
                        ::morok_style::Declaration::Margin(
                            ::morok_style::Side::Right,
                            ::morok_style::LengthPercentageAuto::LengthPercentage(::morok_style::LengthPercentage {
                                pixels: ::morok_style::Pixels(0.0),
                                fraction: ::core::option::Option::None,
                                tokens: ::std::borrow::Cow::Borrowed(
                                    &[
                                        ::morok_style::TokenTerm {
                                            token: ::morok_style::Token::new(0),
                                            factor: 1.0,
                                        },
                                    ],
                                ),
                            }),
                        ),
                        ::morok_style::Declaration::Margin(
                            ::morok_style::Side::Bottom,
                            ::morok_style::LengthPercentageAuto::LengthPercentage(::morok_style::LengthPercentage {
                                pixels: ::morok_style::Pixels(0.0),
                                fraction: ::core::option::Option::None,
                                tokens: ::std::borrow::Cow::Borrowed(&[]),
                            }),
                        ),
                        ::morok_style::Declaration::Margin(
                            ::morok_style::Side::Left,
                            ::morok_style::LengthPercentageAuto::LengthPercentage(::morok_style::LengthPercentage {
                                pixels: ::morok_style::Pixels(0.0),
                                fraction: ::core::option::Option::None,
                                tokens: ::std::borrow::Cow::Borrowed(
                                    &[
                                        ::morok_style::TokenTerm {
                                            token: ::morok_style::Token::new(0),
                                            factor: 1.0,
                                        },
                                    ],
                                ),
                            }),
                        ),
                    ],
                ),
            },
            ::morok_style::StyleRule {
                selector: ::morok_style::Selector {
                    subject: ::morok_style::CompoundSelector {
                        classes: ::std::borrow::Cow::Borrowed(
                            &[::morok_style::Class::new(4)],
                        ),
                        states: ::morok_style::States::NONE,
                    },
                    parent: ::core::option::Option::Some(::morok_style::CompoundSelector {
                        classes: ::std::borrow::Cow::Borrowed(
                            &[::morok_style::Class::new(3)],
                        ),
                        states: ::morok_style::States::NONE,
                    }),
                },
                declarations: ::std::borrow::Cow::Borrowed(
                    &[
                        ::morok_style::Declaration::Position(
                            ::morok_style::Position::Absolute,
                        ),
                        ::morok_style::Declaration::Inset(
                            ::morok_style::Side::Top,
                            ::morok_style::LengthPercentageAuto::LengthPercentage(::morok_style::LengthPercentage {
                                pixels: ::morok_style::Pixels(0.0),
                                fraction: ::core::option::Option::None,
                                tokens: ::std::borrow::Cow::Borrowed(
                                    &[
                                        ::morok_style::TokenTerm {
                                            token: ::morok_style::Token::new(1),
                                            factor: 1.0,
                                        },
                                    ],
                                ),
                            }),
                        ),
                        ::morok_style::Declaration::Inset(
                            ::morok_style::Side::Right,
                            ::morok_style::LengthPercentageAuto::LengthPercentage(::morok_style::LengthPercentage {
                                pixels: ::morok_style::Pixels(0.0),
                                fraction: ::core::option::Option::None,
                                tokens: ::std::borrow::Cow::Borrowed(
                                    &[
                                        ::morok_style::TokenTerm {
                                            token: ::morok_style::Token::new(1),
                                            factor: 1.0,
                                        },
                                    ],
                                ),
                            }),
                        ),
                        ::morok_style::Declaration::Inset(
                            ::morok_style::Side::Bottom,
                            ::morok_style::LengthPercentageAuto::Auto,
                        ),
                        ::morok_style::Declaration::Inset(
                            ::morok_style::Side::Left,
                            ::morok_style::LengthPercentageAuto::Auto,
                        ),
                        ::morok_style::Declaration::GridColumnStart(
                            ::morok_style::GridLine::Line(2),
                        ),
                        ::morok_style::Declaration::GridColumnEnd(
                            ::morok_style::GridLine::Span(2),
                        ),
                        ::morok_style::Declaration::GridRowStart(
                            ::morok_style::GridLine::Line(-1),
                        ),
                        ::morok_style::Declaration::GridRowEnd(
                            ::morok_style::GridLine::Auto,
                        ),
                        ::morok_style::Declaration::JustifySelf(
                            ::morok_style::SelfAlignment::Item(
                                ::morok_style::ItemAlignment::End,
                            ),
                        ),
                        ::morok_style::Declaration::AlignSelf(
                            ::morok_style::SelfAlignment::Auto,
                        ),
                        ::morok_style::Declaration::FlexDirection(
                            ::morok_style::FlexDirection::Column,
                        ),
                        ::morok_style::Declaration::FlexWrap(
                            ::morok_style::FlexWrap::Wrap,
                        ),
                        ::morok_style::Declaration::BackgroundColor(
                            ::morok_style::ColorValue::Transparent,
                        ),
                        ::morok_style::Declaration::BackgroundImage(
                            ::morok_style::BackgroundImage::Gradient(
                                ::morok_style::Gradient::Radial(::morok_style::RadialGradient {
                                    shape: ::morok_style::RadialShape::Circle,
                                    extent: ::morok_style::RadialExtent::ClosestSide,
                                    center_x: ::morok_style::LengthPercentage {
                                        pixels: ::morok_style::Pixels(0.0),
                                        fraction: ::core::option::Option::Some(0.25),
                                        tokens: ::std::borrow::Cow::Borrowed(&[]),
                                    },
                                    center_y: ::morok_style::LengthPercentage {
                                        pixels: ::morok_style::Pixels(0.0),
                                        fraction: ::core::option::Option::Some(0.0),
                                        tokens: ::std::borrow::Cow::Borrowed(&[]),
                                    },
                                    stops: ::std::borrow::Cow::Borrowed(
                                        &[
                                            ::morok_style::ColorStop {
                                                color: ::morok_style::ColorValue::Token(
                                                    ::morok_style::Token::new(14),
                                                ),
                                                position: ::core::option::Option::None,
                                            },
                                            ::morok_style::ColorStop {
                                                color: ::morok_style::ColorValue::CurrentColor,
                                                position: ::core::option::Option::Some(::morok_style::LengthPercentage {
                                                    pixels: ::morok_style::Pixels(12.0),
                                                    fraction: ::core::option::Option::None,
                                                    tokens: ::std::borrow::Cow::Borrowed(&[]),
                                                }),
                                            },
                                        ],
                                    ),
                                }),
                            ),
                        ),
                        ::morok_style::Declaration::BorderWidth(
                            ::morok_style::Side::Bottom,
                            ::morok_style::Length {
                                pixels: ::morok_style::Pixels(2.0),
                                tokens: ::std::borrow::Cow::Borrowed(&[]),
                            },
                        ),
                        ::morok_style::Declaration::BorderStyle(
                            ::morok_style::Side::Bottom,
                            ::morok_style::LineStyle::Solid,
                        ),
                        ::morok_style::Declaration::BorderColor(
                            ::morok_style::Side::Bottom,
                            ::morok_style::ColorValue::CurrentColor,
                        ),
                    ],
                ),
            },
            ::morok_style::StyleRule {
                selector: ::morok_style::Selector {
                    subject: ::morok_style::CompoundSelector {
                        classes: ::std::borrow::Cow::Borrowed(
                            &[::morok_style::Class::new(3)],
                        ),
                        states: ::morok_style::States::FOCUS_WITHIN,
                    },
                    parent: ::core::option::Option::None,
                },
                declarations: ::std::borrow::Cow::Borrowed(
                    &[
                        ::morok_style::Declaration::BackgroundImage(
                            ::morok_style::BackgroundImage::Gradient(
                                ::morok_style::Gradient::Linear(::morok_style::LinearGradient {
                                    direction: ::morok_style::GradientDirection::Angle(45.0),
                                    stops: ::std::borrow::Cow::Borrowed(
                                        &[
                                            ::morok_style::ColorStop {
                                                color: ::morok_style::ColorValue::Token(
                                                    ::morok_style::Token::new(14),
                                                ),
                                                position: ::core::option::Option::None,
                                            },
                                            ::morok_style::ColorStop {
                                                color: ::morok_style::ColorValue::Token(
                                                    ::morok_style::Token::new(1),
                                                ),
                                                position: ::core::option::Option::None,
                                            },
                                        ],
                                    ),
                                }),
                            ),
                        ),
                    ],
                ),
            },
            ::morok_style::StyleRule {
                selector: ::morok_style::Selector {
                    subject: ::morok_style::CompoundSelector {
                        classes: ::std::borrow::Cow::Borrowed(
                            &[::morok_style::Class::new(0), ::morok_style::Class::new(1)],
                        ),
                        states: ::morok_style::States::HOVER,
                    },
                    parent: ::core::option::Option::None,
                },
                declarations: ::std::borrow::Cow::Borrowed(
                    &[
                        ::morok_style::Declaration::BackgroundColor(
                            ::morok_style::ColorValue::Token(
                                ::morok_style::Token::new(12),
                            ),
                        ),
                    ],
                ),
            },
            ::morok_style::StyleRule {
                selector: ::morok_style::Selector {
                    subject: ::morok_style::CompoundSelector {
                        classes: ::std::borrow::Cow::Borrowed(
                            &[::morok_style::Class::new(0), ::morok_style::Class::new(1)],
                        ),
                        states: ::morok_style::States::ACTIVE,
                    },
                    parent: ::core::option::Option::None,
                },
                declarations: ::std::borrow::Cow::Borrowed(
                    &[
                        ::morok_style::Declaration::BackgroundColor(
                            ::morok_style::ColorValue::Token(
                                ::morok_style::Token::new(12),
                            ),
                        ),
                    ],
                ),
            },
        ],
    ),
    classes: ::std::borrow::Cow::Borrowed(
        &[
            ::morok_style::ClassStyle {
                name: ::std::borrow::Cow::Borrowed("button"),
                rules: ::std::borrow::Cow::Borrowed(&[0, 2, 3, 4, 5, 6, 9, 10]),
            },
            ::morok_style::ClassStyle {
                name: ::std::borrow::Cow::Borrowed("primary"),
                rules: ::std::borrow::Cow::Borrowed(&[]),
            },
            ::morok_style::ClassStyle {
                name: ::std::borrow::Cow::Borrowed("toolbar"),
                rules: ::std::borrow::Cow::Borrowed(&[]),
            },
            ::morok_style::ClassStyle {
                name: ::std::borrow::Cow::Borrowed("panel"),
                rules: ::std::borrow::Cow::Borrowed(&[1, 8]),
            },
            ::morok_style::ClassStyle {
                name: ::std::borrow::Cow::Borrowed("badge"),
                rules: ::std::borrow::Cow::Borrowed(&[7]),
            },
        ],
    ),
    keyframes: ::std::borrow::Cow::Borrowed(&[]),
};
