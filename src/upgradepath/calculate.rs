use custom_logger::Logging;
use semver::Version;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Vec<u32>>,
    pub conditional_edges: Vec<ConditionalEdge>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub version: String,
    pub payload: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionalEdge {
    pub edges: Vec<Edge>,
    pub risks: Vec<Risk>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Risk {
    pub url: String,
    pub name: String,
    pub message: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Edge {
    pub from: String,
    pub to: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpgradeResult {
    pub version: String,
    pub image: String,
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            nodes: vec![],
            edges: vec![],
            conditional_edges: vec![],
        }
    }

    // parse the json for graphdata
    pub fn parse_json_graphdata(&self, data: String) -> Result<Self, Box<dyn std::error::Error>> {
        // Parse the string of data into serde_json::ManifestSchema.
        let graph: Graph = serde_json::from_str(&data)?;
        Ok(graph)
    }

    // calculate the upgradepath
    pub fn get_upgrade_path(
        &self,
        log: &Logging,
        from_version: String,
        to_version: String,
        graphdata: Graph,
    ) -> Vec<UpgradeResult> {
        // get ConditionalEdge
        let mut to: Vec<Version> = vec![];
        let mut risks: Vec<Risk> = vec![];
        let mut upgrade_images: Vec<UpgradeResult> = vec![];

        for edge in graphdata.conditional_edges.iter() {
            for e in edge.edges.iter() {
                if e.from == from_version {
                    let version = Version::parse(&e.to).unwrap();
                    to.push(version);
                    for r in edge.risks.iter() {
                        risks.push(r.clone());
                    }
                }
            }
        }

        to.sort();
        log.lo(&format!("list : {:#?}", to.len()));

        if to.len() == 0 {
            return vec![UpgradeResult {
                version: "".to_string(),
                image: "".to_string(),
            }];
        }

        let last_version = to[to.len() - 1].to_string();

        // find the index of the node with version of the intermediate (last_version) if it exists
        let idx = graphdata
            .nodes
            .iter()
            .position(|x| x.version == last_version.to_string());

        let index: u32;

        // find the index of the to_version
        if idx.is_none() {
            index = graphdata
                .nodes
                .iter()
                .position(|x| x.version == to_version)
                .unwrap() as u32;
        } else {
            index = idx.unwrap() as u32;
        }

        // find the head
        let head = graphdata
            .nodes
            .iter()
            .map(|x| Version::parse(&x.version).unwrap())
            .max()
            .unwrap();

        let mut upgrade_list = graphdata
            .edges
            .iter()
            .filter(|x| x[0] == index)
            .map(|x| Version::parse(&graphdata.nodes[x[1] as usize].version).unwrap())
            .collect::<Vec<Version>>();

        upgrade_list.push(Version::parse(&last_version).unwrap());
        log.debug(&format!("upgrade list {:#?}", upgrade_list));

        // check risks and update to exclude vector
        let mut to_exclude: Vec<Version> = vec![];

        for edges in graphdata.conditional_edges.iter() {
            for edge in edges.edges.iter() {
                // work out risks
                if edge.from == from_version && edge.to == last_version {
                    for risk in edges.risks.iter() {
                        log.lo(&format!("risk name    : {:#?}", risk.name));
                        log.lo(&format!("risk message : {:#?}", risk.message));
                    }
                }
                // iterate upgrade_list to see if there is a path to the head
                for item in upgrade_list.iter() {
                    if edge.from == item.to_string() && edge.to == head.to_string() {
                        to_exclude.insert(0, item.clone())
                    }
                }
            }
        }

        let exclude_head = to_exclude.iter().max().unwrap().clone();
        let head_pos = to_exclude
            .iter()
            .position(|x| x.eq(&exclude_head.clone()))
            .unwrap();
        to_exclude.remove(head_pos.clone());
        log.debug(&format!("exclude head version {:#?}", exclude_head));
        log.debug(&format!("version/s to exclude {:#?}", to_exclude));
        upgrade_list.push(Version::parse(&from_version).unwrap());
        upgrade_list.push(head);

        for rm in to_exclude.iter() {
            let pos = upgrade_list.iter().position(|x| x.eq(rm)).unwrap();
            upgrade_list.remove(pos);
        }

        log.info(&format!("upgrade list {:#?}", upgrade_list));

        // finally look up the image references (for v3)
        for node in graphdata.nodes.iter() {
            for version in upgrade_list.iter() {
                if node.version == version.to_string() {
                    match &node.payload {
                        Some(image) => {
                            let upgrade_result = UpgradeResult {
                                version: version.to_string(),
                                image: image.clone(),
                            };
                            upgrade_images.push(upgrade_result);
                        }
                        None => {
                            log.lo("no image found");
                        }
                    }
                }
            }
        }
        upgrade_images.sort_by(|a, b| a.version.cmp(&b.version));
        upgrade_images
    }
}
