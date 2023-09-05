use base64::Engine;
use blake2::{digest::consts::U8, Blake2b, Digest};
use std::{borrow::Cow, collections::HashMap, fmt::Display, num::NonZeroUsize};

type Blake2b64 = Blake2b<U8>;

#[derive(Debug)]
pub struct LogCluster<'a> {
    template_tokens: Vec<Token<'a>>,
    match_count: usize,
    cluster_id: usize,
}

impl<'a> LogCluster<'a> {
    pub const fn match_count(&self) -> usize {
        self.match_count
    }

    /// A global cluster ID that will (hopefully) converge across multiple
    /// instances of the algorithm running on the same log stream.
    pub fn cluster_id(&self) -> String {
        let mut hasher = Blake2b64::new();
        self.template_tokens
            .iter()
            .for_each(|token| token.hash(&mut hasher));
        base64::engine::general_purpose::STANDARD_NO_PAD.encode(hasher.finalize())
    }

    fn new(cluster_id: usize, tokens: Vec<&str>) -> Self {
        Self {
            template_tokens: tokens
                .iter()
                .map(|token| Token::Value(Cow::Owned(token.to_string())))
                .collect(),
            match_count: 1,
            cluster_id,
        }
    }

    // Find the similarity between the log's tokens and this templates tokens
    // from on the range of 0.0 (least similar) to 1.0 (most similar).
    fn seq_dist(&self, tokens: &Tokens) -> (f64, usize) {
        assert!(self.template_tokens.len() == tokens.len());

        if tokens.is_empty() {
            return (1.0, 0);
        }

        let mut sim_count = 0;
        let mut param_count = 0;

        for (token1, token2) in self.template_tokens.iter().zip(tokens.iter()) {
            match token1 {
                Token::Wildcard => {
                    param_count += 1;
                }
                Token::Value(token1) => {
                    if token1 == token2 {
                        sim_count += 1;
                    }
                }
            }
        }

        (
            sim_count as f64 / self.template_tokens.len() as f64,
            param_count,
        )
    }

    // Update template tokens to wildcards where the log tokens and the template
    // tokens are different. Also, return a value for each wildcard tokens.
    fn maybe_update(&mut self, tokens: &Tokens, values: &mut Vec<String>) -> bool {
        assert!(self.template_tokens.len() == tokens.len());
        let mut updated = false;
        for (template_token1, token2) in self.template_tokens.iter_mut().zip(tokens.iter()) {
            match template_token1 {
                Token::Wildcard => values.push(token2.to_string()),
                Token::Value(token1) => {
                    if token1 != token2 {
                        values.push(token2.to_string());
                        *template_token1 = Token::Wildcard;
                        updated = true;
                    }
                }
            }
        }
        updated
    }
}

impl<'a> Display for LogCluster<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for token in &self.template_tokens {
            if !first {
                write!(f, " ")?;
            }
            write!(f, "{ }", token)?;
            first = false;
        }
        Ok(())
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
enum Token<'a> {
    Wildcard,
    Value(Cow<'a, str>),
}

impl<'a> Token<'a> {
    // DRAIN parametrizes on tokens with numbers because they're likely to
    // vary.
    fn has_numbers(&self) -> bool {
        match self {
            Token::Wildcard => false,
            Token::Value(s) => s.chars().any(char::is_numeric),
        }
    }

    fn hash(&self, hasher: &mut Blake2b64) {
        match self {
            Token::Wildcard => {
                hasher.update(1u8.to_le_bytes());
            }
            Token::Value(s) => {
                hasher.update(2u8.to_le_bytes());
                hasher.update(s.as_bytes());
            }
        }
    }
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Wildcard => write!(f, "<*>"),
            Token::Value(value) => write!(f, "{}", value),
        }
    }
}

#[derive(Debug)]
struct Node<'a> {
    children: HashMap<Token<'a>, Node<'a>>,
    cluster_ids: Vec<usize>,
}

impl<'a> Node<'a> {
    fn new() -> Self {
        Self {
            children: HashMap::new(),
            cluster_ids: Vec::new(),
        }
    }
}

type Tokens<'a> = Vec<&'a str>;

pub enum LogUpdateStatus {
    CreatedCluster,
    ChangedClusterTemplate,
    None,
}

pub struct LogParser<'a> {
    first_level: HashMap<usize, Node<'a>>, // First level keyed by the number of tokens in the log
    clusters: lru::LruCache<usize, LogCluster<'a>>, // Clusters stored in a cache by cluster ID
    clusters_count: usize,
    sim_threshold: f64,
    max_node_depth: usize,
    max_children: usize,
    parameterize_numeric_tokens: bool,
    extra_delimiters: Vec<char>,
}

impl<'a> LogParser<'a> {
    pub fn new(max_clusters: NonZeroUsize) -> Self {
        Self {
            first_level: HashMap::new(),
            clusters: lru::LruCache::new(max_clusters),
            clusters_count: 0,
            sim_threshold: 0.4,
            max_node_depth: 4,
            max_children: 100,
            parameterize_numeric_tokens: true,
            extra_delimiters: Vec::new(),
        }
    }

    pub const fn sim_threshold(mut self, value: f64) -> Self {
        self.sim_threshold = value;
        self
    }

    pub const fn max_node_depth(mut self, value: usize) -> Self {
        self.max_node_depth = value;
        self
    }

    pub const fn max_children(mut self, value: usize) -> Self {
        self.max_children = value;
        self
    }

    #[allow(dead_code)]
    pub const fn parameterize_numeric_tokens(mut self, value: bool) -> Self {
        self.parameterize_numeric_tokens = value;
        self
    }

    #[allow(dead_code)]
    pub fn extra_delimiters(mut self, value: Vec<char>) -> Self {
        self.extra_delimiters = value;
        self
    }

    pub fn add_log_line(
        &mut self,
        line: &str,
        values: &mut Vec<String>,
    ) -> (&LogCluster, LogUpdateStatus) {
        let tokens = tokenize(line, &self.extra_delimiters);

        values.clear();

        match self.tree_search(&tokens) {
            None => {
                // No cluster found for the log line, create a new cluster.
                self.clusters_count += 1;
                let cluster_id = self.clusters_count;
                let cluster = LogCluster::new(cluster_id, tokens);
                self.add_seq_to_prefix_tree(&cluster); // Add the node path to the new cluster.
                (
                    self.clusters.get_or_insert(cluster_id, || cluster),
                    LogUpdateStatus::CreatedCluster,
                )
            }
            Some(cluster_id) => {
                // Existing cluster found for the log line, update it if there are differences.
                let cluster = self.clusters.get_mut(&cluster_id).unwrap(); // Already verified this cluster exists in tree_search()
                cluster.match_count += 1;
                let updated = cluster.maybe_update(&tokens, values);
                (
                    cluster,
                    if updated {
                        LogUpdateStatus::ChangedClusterTemplate
                    } else {
                        LogUpdateStatus::None
                    },
                )
            }
        }
    }

    // Update the prefix tree with the new cluster creating a new node path where necessary.
    fn add_seq_to_prefix_tree(&mut self, cluster: &LogCluster<'a>) {
        let token_count = cluster.template_tokens.len();
        let mut curr_node = self
            .first_level
            .entry(token_count)
            .or_insert_with(Node::new);

        if token_count == 0 {
            curr_node.cluster_ids.push(cluster.cluster_id);
            return;
        }

        let mut curr_node_depth = 1;
        for token in &cluster.template_tokens {
            if curr_node_depth >= self.max_node_depth || curr_node_depth >= token_count {
                break;
            }

            if curr_node.children.contains_key(token) {
                curr_node = curr_node.children.get_mut(token).unwrap();
            } else if self.parameterize_numeric_tokens && token.has_numbers() {
                curr_node = curr_node
                    .children
                    .entry(Token::Wildcard)
                    .or_insert_with(Node::new);
            } else if curr_node.children.contains_key(&Token::Wildcard) {
                if curr_node.children.len() < self.max_children {
                    curr_node = curr_node
                        .children
                        .entry(token.clone())
                        .or_insert_with(Node::new);
                } else {
                    curr_node.children.get_mut(&Token::Wildcard).unwrap();
                }
            } else if curr_node.children.len() + 1 < self.max_children {
                // We leave space for a final wildcard token
                curr_node = curr_node
                    .children
                    .entry(token.clone())
                    .or_insert_with(Node::new);
            } else if curr_node.children.len() + 1 == self.max_children {
                // Add a wildcard token as the last child of a cluster group
                curr_node = curr_node
                    .children
                    .entry(Token::Wildcard)
                    .or_insert_with(Node::new);
            } else {
                unreachable!();
            }

            curr_node_depth += 1;
        }

        // Add new cluster to the leaf node
        let cluster_id = cluster.cluster_id;
        let mut new_cluster_ids = Vec::new();
        for cluster_id in &curr_node.cluster_ids {
            if self.clusters.contains(cluster_id) {
                new_cluster_ids.push(*cluster_id);
            }
        }
        new_cluster_ids.push(cluster_id);
        curr_node.cluster_ids = new_cluster_ids;
    }

    fn tree_search(&mut self, tokens: &Tokens) -> Option<usize> {
        let token_count = tokens.len();

        let mut curr_node = self.first_level.get(&token_count);

        let mut curr_node_depth = 1;
        for token in tokens {
            if curr_node_depth >= self.max_node_depth {
                break;
            }
            if curr_node_depth == token_count {
                break;
            }

            match curr_node {
                None => break,
                Some(node) => {
                    curr_node = node.children.get(&Token::Value(Cow::Borrowed(token)));
                    if curr_node.is_none() {
                        // No match found so attempt a wildcard token
                        curr_node = node.children.get(&Token::Wildcard);
                    }
                }
            }

            curr_node_depth += 1
        }

        match curr_node {
            None => None,
            Some(node) => {
                // Find the best match by finding the maximum similarity
                let mut max_sim = 0.0;
                let mut max_param_count = 0;
                let mut max_cluster_id: Option<usize> = None;

                for cluster_id in &node.cluster_ids {
                    let cluster = self.clusters.get(cluster_id);
                    match cluster {
                        None => continue,
                        Some(cluster) => {
                            let (sim, param_count) = cluster.seq_dist(tokens);
                            if sim > max_sim || (sim == max_sim && param_count > max_param_count) {
                                max_sim = sim;
                                max_param_count = param_count;
                                max_cluster_id = Some(*cluster_id);
                            }
                        }
                    }
                }

                if max_sim >= self.sim_threshold {
                    max_cluster_id
                } else {
                    None
                }
            }
        }
    }
}

fn tokenize<'a>(s: &'a str, extra_delimiters: &[char]) -> Tokens<'a> {
    s.split(|c: char| c.is_whitespace() || extra_delimiters.contains(&c))
        .filter(|s| !s.is_empty())
        .collect::<Tokens<'a>>()
}

#[cfg(test)]
mod tests {
    use std::{num::NonZeroUsize, vec};

    use super::{tokenize, LogParser};

    #[test]
    fn add_log_line() {
        let lines = vec![
            "Dec 10 07:07:38 LabSZ sshd[24206]: input_userauth_request: invalid user test9 [preauth]",
            "Dec 10 07:08:28 LabSZ sshd[24208]: input_userauth_request: invalid user webmaster [preauth]",
            "Dec 10 09:12:32 LabSZ sshd[24490]: Failed password for invalid user ftpuser from 0.0.0.0 port 62891 ssh2",
            "Dec 10 09:12:35 LabSZ sshd[24492]: Failed password for invalid user pi from 0.0.0.0 port 49289 ssh2",
            "Dec 10 09:12:44 LabSZ sshd[24501]: Failed password for invalid user ftpuser from 0.0.0.0 port 60836 ssh2",
            "Dec 10 07:28:03 LabSZ sshd[24245]: input_userauth_request: invalid user pgadmin [preauth]",
        ];

        let expected = vec![
            "Dec 10 07:07:38 LabSZ sshd[24206]: input_userauth_request: invalid user test9 [preauth]",
            "Dec 10 <*> LabSZ <*> input_userauth_request: invalid user <*> [preauth]",
            "Dec 10 09:12:32 LabSZ sshd[24490]: Failed password for invalid user ftpuser from 0.0.0.0 port 62891 ssh2",
            "Dec 10 <*> LabSZ <*> Failed password for invalid user <*> from 0.0.0.0 port <*> ssh2",
            "Dec 10 <*> LabSZ <*> Failed password for invalid user <*> from 0.0.0.0 port <*> ssh2",
            "Dec 10 <*> LabSZ <*> input_userauth_request: invalid user <*> [preauth]",
        ];

        let expected_values = vec![
            vec![],
            vec!["07:08:28", "sshd[24208]:", "webmaster"],
            vec![],
            vec!["09:12:35", "sshd[24492]:", "pi", "49289"],
            vec!["09:12:44", "sshd[24501]:", "ftpuser", "60836"],
            vec!["07:28:03", "sshd[24245]:", "pgadmin"],
        ];

        let mut parser = LogParser::new(NonZeroUsize::new(1000).unwrap());
        for ((line, expected), expected_values) in lines
            .iter()
            .zip(expected.iter())
            .zip(expected_values.iter())
        {
            let mut values = Vec::new();
            let (group, _) = parser.add_log_line(line, &mut values);
            let actual = format!("{}", group);
            assert_eq!(expected.to_string(), actual);
            assert_eq!(
                expected_values
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>(),
                values
            );
        }
    }

    #[test]
    fn add_log_line_sim_75() {
        let lines = vec![
            "Dec 10 07:07:38 LabSZ sshd[24206]: input_userauth_request: invalid user test9 [preauth]",
            "Dec 10 07:08:28 LabSZ sshd[24208]: input_userauth_request: invalid user webmaster [preauth]",
            "Dec 10 09:12:32 LabSZ sshd[24490]: Failed password for invalid user ftpuser from 0.0.0.0 port 62891 ssh2",
            "Dec 10 09:12:35 LabSZ sshd[24492]: Failed password for invalid user pi from 0.0.0.0 port 49289 ssh2",
            "Dec 10 09:12:44 LabSZ sshd[24501]: Failed password for invalid user ftpuser from 0.0.0.0 port 60836 ssh2",
            "Dec 10 07:28:03 LabSZ sshd[24245]: input_userauth_request: invalid user pgadmin [preauth]",
        ];

        let expected = vec![
            "Dec 10 07:07:38 LabSZ sshd[24206]: input_userauth_request: invalid user test9 [preauth]",
            "Dec 10 07:08:28 LabSZ sshd[24208]: input_userauth_request: invalid user webmaster [preauth]",
            "Dec 10 09:12:32 LabSZ sshd[24490]: Failed password for invalid user ftpuser from 0.0.0.0 port 62891 ssh2",
            "Dec 10 <*> LabSZ <*> Failed password for invalid user <*> from 0.0.0.0 port <*> ssh2",
            "Dec 10 <*> LabSZ <*> Failed password for invalid user <*> from 0.0.0.0 port <*> ssh2",
            "Dec 10 07:28:03 LabSZ sshd[24245]: input_userauth_request: invalid user pgadmin [preauth]"
        ];

        let expected_values = vec![
            vec![],
            vec![],
            vec![],
            vec!["09:12:35", "sshd[24492]:", "pi", "49289"],
            vec!["09:12:44", "sshd[24501]:", "ftpuser", "60836"],
            vec![],
        ];

        let mut parser = LogParser::new(NonZeroUsize::new(1000).unwrap())
            .sim_threshold(0.75)
            .max_node_depth(4);
        for ((line, expected), expected_values) in lines
            .iter()
            .zip(expected.iter())
            .zip(expected_values.iter())
        {
            let mut values = Vec::new();
            let (group, _) = parser.add_log_line(line, &mut values);
            let actual = format!("{}", group);
            assert_eq!(expected.to_string(), actual);
            assert_eq!(
                expected_values
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>(),
                values
            );
        }
    }

    #[test]
    fn max_clusters() {
        let mut parser = LogParser::new(NonZeroUsize::new(1).unwrap()); // Only one max cluster

        let lines = vec![
            "A format 1",
            "A format 2",
            "B format 1",
            "B format 2",
            "A format 3",
        ];

        let expected = vec![
            "A format 1",
            "A format <*>",
            "B format 1",
            "B format <*>",
            "A format 3",
        ];

        for (line, expected) in lines.iter().zip(expected.iter()) {
            let (group, _) = parser.add_log_line(line, &mut Vec::new());
            let actual = format!("{}", group);
            assert_eq!(expected.to_string(), actual);
        }
    }

    #[test]
    fn tokens() {
        let tokens = tokenize(
            "    $$$$a    b$$$$c   $$$$abc---- def\t\nxyz\t  --$$",
            &['$', '-'],
        );
        assert_eq!(tokens, vec!["a", "b", "c", "abc", "def", "xyz"]);
    }
}
