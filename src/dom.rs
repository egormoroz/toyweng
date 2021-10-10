use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
pub struct Node<'a> {
    pub node_type: NodeType<'a>,
    pub children: Vec<Node<'a>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum NodeType<'a> {
    Text(&'a str),
    Element(ElementData<'a>),
}

pub type AttrMap<'a> = HashMap<&'a str, &'a str>;

#[derive(Debug, PartialEq, Eq)]
pub struct ElementData<'a> {
    pub tag_name: &'a str,
    pub attributes: AttrMap<'a>,
}

pub fn text<'a>(data: &'a str) -> Node<'a> {
    Node { 
        children: vec![], 
        node_type: NodeType::Text(data)
    }
}

pub fn elem<'a>(name: &'a str, attrs: AttrMap<'a>, 
    children: Vec<Node<'a>>) -> Node<'a> 
{
    Node {
        children,
        node_type: NodeType::Element(ElementData {
            tag_name: name,
            attributes: attrs
        })
    }
}
