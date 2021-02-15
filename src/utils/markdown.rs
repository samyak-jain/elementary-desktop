use comrak::{
    nodes::{AstNode, NodeValue},
    parse_document, Arena, ComrakOptions,
};
use iced::Text;

pub fn parse_text(content: String) {
    let arena = Arena::new();
    let root = parse_document(&arena, &content, &ComrakOptions::default());

    let message_contents = iter_nodes(root, &|node, mut message_contents| {
        match node.data.clone().into_inner().value {
            NodeValue::Text(text) => {
                message_contents.push(String::from_utf8(text).unwrap());
            }
            NodeValue::Document => {}
            NodeValue::BlockQuote => {}
            NodeValue::List(_) => {}
            NodeValue::Item(_) => {}
            NodeValue::DescriptionList => {}
            NodeValue::DescriptionItem(_) => {}
            NodeValue::DescriptionTerm => {}
            NodeValue::DescriptionDetails => {}
            NodeValue::CodeBlock(code_block) => {
                if code_block.fenced {
                    if code_block.fence_length == 3 {
                        message_contents.push(String::from_utf8(code_block.literal).unwrap());
                    } else {
                        message_contents[message_contents.len() - 1]
                    }
                }
            }
            NodeValue::HtmlBlock(_) => {}
            NodeValue::Paragraph => {}
            NodeValue::Heading(_) => {}
            NodeValue::ThematicBreak => {}
            NodeValue::FootnoteDefinition(_) => {}
            NodeValue::Table(_) => {}
            NodeValue::TableRow(_) => {}
            NodeValue::TableCell => {}
            NodeValue::TaskItem(_) => {}
            NodeValue::SoftBreak => {}
            NodeValue::LineBreak => {}
            NodeValue::Code(_) => {}
            NodeValue::HtmlInline(_) => {}
            NodeValue::Emph => {}
            NodeValue::Strong => {}
            NodeValue::Strikethrough => {}
            NodeValue::Superscript => {}
            NodeValue::Link(_) => {}
            NodeValue::Image(_) => {}
            NodeValue::FootnoteReference(_) => {}
        };
        message_contents
    });
}

fn iter_nodes<'a, F>(node: &'a AstNode<'a>, element_builder: &F) -> Vec<String>
where
    F: Fn(&'a AstNode<'a>, Vec<String>) -> Vec<String>,
{
    let mut message_contents = Vec::new();
    message_contents = element_builder(node, message_contents);
    for c in node.children() {
        message_contents = iter_nodes(c, element_builder);
    }

    message_contents
}
