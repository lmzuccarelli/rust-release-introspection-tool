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
        log.lo(&format!("list length : {}", to.len()));
        log.trace(&format!("list        : {:#?}", to));

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
        let from_index = graphdata
            .nodes
            .iter()
            .position(|x| x.version == from_version)
            .unwrap() as u32;

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

        log.trace(&format!("index of from_version {}", from_index));
        log.trace(&format!("index of to_version {}", index));

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

        // find the head position
        let head_pos = graphdata
            .nodes
            .iter()
            .position(|x| x.version == head.to_string())
            .unwrap();
        log.debug(&format!("head_pos {}", head_pos));

        // check if there is a path for each remaining node to the head
        for rm in upgrade_list.clone().iter() {
            let current_idx = graphdata
                .nodes
                .iter()
                .position(|x| x.version == rm.to_string())
                .unwrap() as u32;

            log.debug(&format!("current_idx {}", current_idx));
            let path_exists = graphdata
                .edges
                .iter()
                .filter(|x| x[0] == current_idx && x[1] == head_pos as u32)
                .count();

            log.debug(&format!("path exists {} {}", path_exists, current_idx));

            if path_exists > 0 {
                let pos = upgrade_list.iter().position(|x| x.eq(rm)).unwrap();
                upgrade_list.remove(pos);
            }
        }
        log.trace(&format!("upgrade list {:#?}", upgrade_list));
        //}

        let mut dedup_upgrade_list = vec![];
        for item in upgrade_list.iter() {
            if !dedup_upgrade_list.contains(item) {
                dedup_upgrade_list.insert(0, item.clone());
            }
        }
        log.trace(&format!("dedup upgrade list {:#?}", dedup_upgrade_list));

        // finally look up the image references (for v3)
        for node in graphdata.nodes.iter() {
            for version in dedup_upgrade_list.iter() {
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
        upgrade_images.sort_by(|a, b| {
            Version::parse(&a.version)
                .unwrap()
                .cmp(&Version::parse(&b.version).unwrap())
        });
        log.trace(&format!("sorted final list {:#?}", upgrade_images));
        upgrade_images
    }
}
