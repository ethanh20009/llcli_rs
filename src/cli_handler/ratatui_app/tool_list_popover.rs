use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Margin, Rect},
    style::{Color, Modifier, Style, Stylize, palette::tailwind::SLATE},
    symbols,
    text::Line,
    widgets::{Block, Borders, HighlightSpacing, List, ListItem, Paragraph, StatefulWidget},
};
use strum::IntoEnumIterator;
use tui_popup::{Popup, SizedWrapper};

use crate::provider::LLMTools;

use super::App;

const SELECTED_STYLE: Style = Style::new().bg(Color::Blue).add_modifier(Modifier::BOLD);

#[derive(Debug, Clone, Copy, strum::FromRepr, strum::EnumIter)]
pub(super) enum LlmToolEnum {
    Search,
}

impl LLMTools {
    pub(super) fn toggle(&mut self, item: LlmToolEnum) {
        match item {
            LlmToolEnum::Search => {
                self.search = !self.search;
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct LlmToolItem<'a> {
    llm_item: LlmToolEnum,
    llm_tools: &'a LLMTools,
}

impl LlmToolEnum {
    pub(super) fn with_context<'a>(self, llm_tools: &'a LLMTools) -> LlmToolItem<'a> {
        LlmToolItem {
            llm_item: self,
            llm_tools,
        }
    }
}

impl<'a> From<LlmToolItem<'a>> for ListItem<'a> {
    fn from(value: LlmToolItem<'a>) -> Self {
        let (display_name, activated) = match value.llm_item {
            LlmToolEnum::Search => ("Web Search", value.llm_tools.search),
        };
        match activated {
            true => ListItem::new(format!(" ✓ {}", display_name)).fg(Color::Green),
            false => ListItem::new(format!(" ☐ {}", display_name)),
        }
    }
}

impl<'a, 't> App<'a, 't> {
    pub(super) fn llm_options_popup(&mut self, area: Rect, frame: &mut Frame) {
        let popover_rect = area.inner(Margin::new(5, 5));

        let block = Block::new()
            .title(Line::raw("LLM Options").centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY);

        let item_enum = LlmToolEnum::iter().collect::<Vec<_>>();
        // Iterate through all elements in the `items` and stylize them.
        let items: Vec<ListItem> = item_enum
            .iter()
            .enumerate()
            .map(|(i, item)| ListItem::from(item.with_context(self.provider.flags())))
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        frame.render_stateful_widget(list, popover_rect, &mut self.llm_tool_options_state);
    }
}
