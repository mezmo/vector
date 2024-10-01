use base64::Engine;
use blake2::{digest::consts::U8, Blake2b, Digest};
use lru::LruCache;
use std::{borrow::Cow, collections::HashMap, fmt::Display, num::NonZeroUsize};
use vrl::value::Value;

type Blake2b64 = Blake2b<U8>;

/// An alias for the local id
pub type LocalId = usize;

#[derive(Debug)]
pub struct LogCluster<'a> {
    template_tokens: Vec<Token<'a>>,
    match_count: usize,
    samples: HashMap<String, LogSample>,
    /// The local numeric identifier (auto-incremental)
    id: LocalId,
}

impl<'a> LogCluster<'a> {
    pub const fn match_count(&self) -> usize {
        self.match_count
    }

    pub fn samples_mut(&mut self) -> &mut HashMap<String, LogSample> {
        &mut self.samples
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

    pub const fn local_id(&self) -> LocalId {
        self.id
    }

    fn new(cluster_id: usize, parameterize_numeric_tokens: bool, tokens: Vec<&str>) -> Self {
        Self {
            template_tokens: tokens
                .iter()
                .map(|token| {
                    if parameterize_numeric_tokens && token.chars().any(char::is_numeric) {
                        Token::Wildcard
                    } else {
                        Token::Value(Cow::Owned(token.to_string()))
                    }
                })
                .collect(),
            match_count: 1,
            samples: HashMap::new(),
            id: cluster_id,
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
    fn maybe_update(&mut self, tokens: &Tokens) -> bool {
        assert_eq!(self.template_tokens.len(), tokens.len());
        let mut updated = false;
        for (template_token1, token2) in self.template_tokens.iter_mut().zip(tokens.iter()) {
            match template_token1 {
                Token::Wildcard => {}
                Token::Value(token1) => {
                    if token1 != token2 {
                        *template_token1 = Token::Wildcard;
                        updated = true;
                    }
                }
            }
        }
        updated
    }

    pub fn get_unstored_samples(&self) -> Vec<&LogSample> {
        self.samples
            .iter()
            .filter(|(_, sample)| !sample.is_stored())
            .map(|(_, sample)| sample)
            .collect::<Vec<&LogSample>>()
    }

    fn clear_samples(&mut self) {
        self.samples = HashMap::new();
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

#[derive(PartialEq)]
pub enum LogClusterStatus {
    /// Determines that the template was created or has changed
    ChangedTemplate,
    /// Defines that the log cluster itself was not affected by the event
    None,
}

#[derive(Debug)]
pub struct LogSample {
    pub line: String,
    pub sample: Value,
    stored: bool,
}

impl LogSample {
    const fn new(line: String) -> Self {
        Self {
            line,
            sample: Value::Null,
            stored: false,
        }
    }

    pub fn id(&self) -> String {
        let mut hasher = Blake2b64::new();
        hasher.update(self.line.as_bytes());
        base64::engine::general_purpose::STANDARD_NO_PAD.encode(hasher.finalize())
    }

    pub fn sample(mut self, sample: Value) -> Self {
        self.sample = sample;
        self
    }

    pub const fn is_stored(&self) -> bool {
        self.stored
    }

    pub fn mark_as_stored(&mut self) -> &Self {
        if !self.stored {
            self.stored = true;
            // Once the sample is stored we don't need potentially heavy object in memory.
            self.sample = Value::Null;
        }

        self
    }
}

pub struct LogParser<'a> {
    first_level: HashMap<usize, Node<'a>>, // First level keyed by the number of tokens in the log
    clusters: LruCache<usize, LogCluster<'a>>, // Clusters stored in a cache by cluster ID
    clusters_count: usize,
    samples_counts: HashMap<String, usize>,
    sim_threshold: f64,
    max_node_depth: usize,
    max_children: usize,
    parameterize_numeric_tokens: bool,
    extra_delimiters: Vec<char>,
    max_log_samples_amount: NonZeroUsize,
}

impl<'a> LogParser<'a> {
    pub fn new(max_clusters: NonZeroUsize, max_log_samples_amount: NonZeroUsize) -> Self {
        Self {
            first_level: HashMap::new(),
            clusters: LruCache::new(max_clusters),
            clusters_count: 0,
            samples_counts: HashMap::new(),
            sim_threshold: 0.4,
            max_node_depth: 8,
            max_children: 40,
            parameterize_numeric_tokens: true,
            extra_delimiters: Vec::new(),
            max_log_samples_amount,
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

    pub fn mark_cluster_samples_as_stored(&mut self, cluster_id: usize) {
        if let Some(cluster) = self.clusters.get_mut(&cluster_id) {
            cluster.samples.iter_mut().for_each(|(_, sample)| {
                sample.mark_as_stored();
            });
        }
    }

    pub fn add_log_line(
        &mut self,
        line: &str,
        sample_context: Option<&Value>,
    ) -> (&LogCluster, LogClusterStatus) {
        let tokens = tokenize(line, &self.extra_delimiters);

        let (cluster, cluster_status) = match self.tree_search(&tokens) {
            None => {
                // No cluster found for the log line, create a new cluster.
                self.clusters_count += 1;
                let cluster_id = self.clusters_count;
                let cluster = LogCluster::new(cluster_id, self.parameterize_numeric_tokens, tokens);
                self.add_seq_to_prefix_tree(&cluster); // Add the node path to the new cluster.
                (
                    self.clusters.get_or_insert_mut(cluster_id, || cluster),
                    LogClusterStatus::ChangedTemplate,
                )
            }
            Some(cluster_id) => {
                // Existing cluster found for the log line, update it if there are differences.
                let cluster = self.clusters.get_mut(&cluster_id).unwrap(); // Already verified this cluster exists in tree_search()
                cluster.match_count += 1;
                let updated = cluster.maybe_update(&tokens);
                (
                    cluster,
                    if updated {
                        LogClusterStatus::ChangedTemplate
                    } else {
                        LogClusterStatus::None
                    },
                )
            }
        };

        let cluster_id = cluster.cluster_id();
        let sample_count = *self.samples_counts.get(&cluster_id).unwrap_or(&0);

        if sample_count < self.max_log_samples_amount.into() {
            let log_sample = LogSample::new(line.to_string());
            let log_sample_id = log_sample.id();

            // Add unique only samples
            if let std::collections::hash_map::Entry::Vacant(entry) =
                cluster.samples_mut().entry(log_sample_id.clone())
            {
                // Clone and assign a sample_context to a sample only if
                // the sample is going to be added to a cluster.
                let log_sample = log_sample.sample(sample_context.unwrap_or(&Value::Null).clone());

                entry.insert(log_sample);
                let sample_count = sample_count + 1;
                self.samples_counts.insert(cluster_id, sample_count);
            }
        } else if !cluster.samples.is_empty() && cluster.get_unstored_samples().is_empty() {
            // Clear the samples once the threshold is reached to free up memory.
            // All samples have to be stored.
            cluster.clear_samples();
        }

        (cluster, cluster_status)
    }

    // Update the prefix tree with the new cluster creating a new node path where necessary.
    fn add_seq_to_prefix_tree(&mut self, cluster: &LogCluster<'a>) {
        let token_count = cluster.template_tokens.len();
        let mut curr_node = self
            .first_level
            .entry(token_count)
            .or_insert_with(Node::new);

        if token_count == 0 {
            curr_node.cluster_ids.push(cluster.id);
            return;
        }

        let mut curr_node_depth = 1;
        for token in &cluster.template_tokens {
            // There's a diminishing return in having more children as the height increases
            let max_children = if curr_node_depth > 1 {
                let factor = curr_node_depth * 2;
                let result = self.max_children / factor;
                if result < 2 {
                    2
                } else {
                    result
                }
            } else {
                self.max_children
            };
            if curr_node_depth >= self.max_node_depth || curr_node_depth >= token_count {
                break;
            }

            // Set the next node, inserting where necessary
            curr_node = if curr_node.children.contains_key(token) {
                curr_node.children.get_mut(token).unwrap()
            } else if self.parameterize_numeric_tokens && token.has_numbers() {
                curr_node
                    .children
                    .entry(Token::Wildcard)
                    .or_insert_with(Node::new)
            } else if curr_node.children.contains_key(&Token::Wildcard) {
                if curr_node.children.len() < max_children {
                    curr_node
                        .children
                        .entry(token.clone())
                        .or_insert_with(Node::new)
                } else {
                    curr_node.children.get_mut(&Token::Wildcard).unwrap()
                }
            } else if curr_node.children.len() + 1 < max_children {
                // We leave space for a final wildcard token
                curr_node
                    .children
                    .entry(token.clone())
                    .or_insert_with(Node::new)
            } else if curr_node.children.len() + 1 == max_children {
                // Add a wildcard token as the last child of a cluster group
                curr_node
                    .children
                    .entry(Token::Wildcard)
                    .or_insert_with(Node::new)
            } else {
                unreachable!();
            };

            curr_node_depth += 1;
        }

        // Add new cluster to the leaf node
        let cluster_id = cluster.id;
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
    use super::*;
    use std::collections::{BTreeMap, HashSet};
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
            "Dec <*> <*> LabSZ <*> input_userauth_request: invalid user <*> [preauth]",
            "Dec <*> <*> LabSZ <*> input_userauth_request: invalid user <*> [preauth]",
            "Dec <*> <*> LabSZ <*> Failed password for invalid user ftpuser from <*> port <*> <*>",
            // The next will cause ftpuser to become a wildcard
            "Dec <*> <*> LabSZ <*> Failed password for invalid user <*> from <*> port <*> <*>",
            "Dec <*> <*> LabSZ <*> Failed password for invalid user <*> from <*> port <*> <*>",
            "Dec <*> <*> LabSZ <*> input_userauth_request: invalid user <*> [preauth]",
        ];

        let mut parser = LogParser::new(
            NonZeroUsize::new(1000).unwrap(),
            NonZeroUsize::new(5).unwrap(),
        );
        for (line, expected) in lines.iter().zip(expected.iter()) {
            let (group, _) = parser.add_log_line(line, None);
            let actual = format!("{}", group);
            assert_eq!(expected.to_string(), actual);
        }
    }

    #[test]
    fn add_log_line_sim_75() {
        let lines = vec![
            "Dec 10 07:07:38 LabSZ sshd[24206]: input_userauth_request: invalid user test9 [preauth]",
            "Dec 10 07:08:28 LabSZ sshd[24208]: input_userauth_request: invalid user webmaster [preauth]",
            "Dec 10 09:12:32 LabSZ sshd[24490]: Failed password for invalid user ftpuser from 0.0.0.0 port 62891 ssh2",
            "Dec 10 09:12:35 LabSZ sshd[24492]: Failed password for invalid user pi from 0.0.0.0 port 49289 ssh2",
            "Dec 10 09:12:44 LabSZ sshd[24501]: Failed password for invalid user my_user from 0.0.0.0 port 60836 ssh2",
            "Dec 10 07:28:03 LabSZ sshd[24245]: input_userauth_request: invalid user pgadmin [preauth]",
        ];

        let expected = vec![
            "Dec <*> <*> LabSZ <*> input_userauth_request: invalid user <*> [preauth]",
            "Dec <*> <*> LabSZ <*> input_userauth_request: invalid user webmaster [preauth]",
            "Dec <*> <*> LabSZ <*> Failed password for invalid user ftpuser from <*> port <*> <*>",
            // Note that these 2 will not templatize the group further
            "Dec <*> <*> LabSZ <*> Failed password for invalid user pi from <*> port <*> <*>",
            "Dec <*> <*> LabSZ <*> Failed password for invalid user my_user from <*> port <*> <*>",
            "Dec <*> <*> LabSZ <*> input_userauth_request: invalid user pgadmin [preauth]",
        ];

        let mut parser = LogParser::new(
            NonZeroUsize::new(1000).unwrap(),
            NonZeroUsize::new(5).unwrap(),
        )
        .sim_threshold(0.75)
        .max_node_depth(4);
        for (line, expected) in lines.iter().zip(expected.iter()) {
            let (group, _) = parser.add_log_line(line, None);
            let actual = format!("{}", group);
            assert_eq!(expected.to_string(), actual);
        }
    }

    #[test]
    fn max_clusters() {
        let mut parser =
            LogParser::new(NonZeroUsize::new(1).unwrap(), NonZeroUsize::new(5).unwrap()); // Only one max cluster

        let lines = vec![
            "A format 1",
            "A format 2",
            "B format 1",
            "B format 2",
            "A format 3",
        ];

        let expected = vec![
            ("A format <*>", 1usize),
            ("A format <*>", 1),
            ("B format <*>", 2),
            ("B format <*>", 2),
            ("A format <*>", 3),
        ];

        for (line, expected) in lines.iter().zip(expected.iter()) {
            let (group, _) = parser.add_log_line(line, None);
            let actual = format!("{}", group);
            assert_eq!(expected.0.to_string(), actual);
            assert_eq!(expected.1, group.id);
        }
    }

    #[test]
    fn stable_cluster_id_test() {
        let mut parser = LogParser::new(
            NonZeroUsize::new(100).unwrap(),
            NonZeroUsize::new(5).unwrap(),
        )
        .sim_threshold(0.5)
        .max_node_depth(6);

        let lines = vec![
            "A format 1",
            "A format 2",
            "B format 1",
            "B format 2",
            "A format 3",
            "Something that can be set into a template: my_user",
            "Something that can be set into a template: another_user",
            "Hello my world",
            "Hello my Mars",
        ];

        let expected = vec![
            "A format <*>",
            "A format <*>",
            "B format <*>",
            "B format <*>",
            "A format <*>",
            "Something that can be set into a template: my_user",
            "Something that can be set into a template: <*>",
            "Hello my world",
            "Hello my <*>",
        ];

        let mut cluster_map = HashMap::<usize, HashSet<String>>::new();
        for (line, expected) in lines.iter().zip(expected.iter()) {
            let (group, _) = parser.add_log_line(line, None);
            let actual = format!("{}", group);
            assert_eq!(expected.to_string(), actual);
            let gen_ids = cluster_map.entry(group.id).or_default();
            gen_ids.insert(group.cluster_id());
        }

        assert_eq!(cluster_map.get(&1).unwrap().len(), 1);
        assert_eq!(cluster_map.get(&2).unwrap().len(), 1);
        // The following groups got templatized as part of similitude check
        assert_eq!(cluster_map.get(&3).unwrap().len(), 2);
        assert_eq!(cluster_map.get(&4).unwrap().len(), 2);
    }

    #[test]
    fn several_wildcards_followed_by_token_test() {
        let mut parser = LogParser::new(
            NonZeroUsize::new(100).unwrap(),
            NonZeroUsize::new(5).unwrap(),
        )
        .sim_threshold(0.9)
        .max_node_depth(8)
        .max_children(16);

        let lines = vec![
            "1 x X 0", "1 y Y 0", "1 z Z 0", "1 1 O 0", "1 y Y 0", "1 m M 0", "1 n N 0",
        ];

        for line in lines.iter() {
            let (group, _) = parser.add_log_line(line, None);
            let _actual = format!("{}", group);
        }
        let root = parser.first_level.get(&4).unwrap();
        let first_wildcard = root.children.get(&Token::Wildcard).unwrap();
        assert_eq!(
            first_wildcard.children.len(),
            4,
            "Expected 4 children, max at second level"
        );
        let second_wildcard = first_wildcard.children.get(&Token::Wildcard).unwrap();
        assert_eq!(
            second_wildcard.children.len(),
            2,
            "Expected 2 children to contain O, M and N"
        );
    }

    #[test]
    fn tokens() {
        let tokens = tokenize(
            "    $$$$a    b$$$$c   $$$$abc---- def\t\nxyz\t  --$$",
            &['$', '-'],
        );
        assert_eq!(tokens, vec!["a", "b", "c", "abc", "def", "xyz"]);
    }

    #[test]
    fn collect_samples() {
        let lines = vec![
            "test message 1",
            "test message 1",
            "test message 1",
            "test message 2",
            "test message 2",
            "test message 2",
            "test message N",
            "Something completely different 1",
            "Something completely different 1",
            "Something completely different 3",
            "Something completely different 3",
            "Something completely different 2",
            "Something completely different 2",
            "Something completely different N",
        ];

        let expected = vec![
            "test message 1",
            "test message 2",
            "Something completely different 1",
            "Something completely different 3",
        ];

        let mut parser = LogParser::new(
            NonZeroUsize::new(1000).unwrap(),
            NonZeroUsize::new(2).unwrap(),
        );

        let mut stored_samples: Vec<String> = Vec::new();

        for line in lines.iter() {
            let (group, _) = parser.add_log_line(line, None);

            let samples = group.get_unstored_samples();
            samples
                .iter()
                .for_each(|s| stored_samples.push(s.line.clone()));

            let local_id = group.local_id();
            parser.mark_cluster_samples_as_stored(local_id);
        }

        assert_eq!(stored_samples.len(), expected.len());

        for (line, expected) in stored_samples.iter().zip(expected.iter()) {
            assert_eq!(*line, expected.to_string());
        }
    }

    #[test]
    fn collect_samples_clear() {
        let mut parser = LogParser::new(
            NonZeroUsize::new(1000).unwrap(),
            NonZeroUsize::new(2).unwrap(),
        );

        let sample_context = Value::Object(BTreeMap::from([(
            "message".into(),
            Value::Object(BTreeMap::from([("foo".into(), "bar".into())])),
        )]));

        // This line will be added to samples
        let (group, _) = parser.add_log_line("test message 1", Some(&sample_context));
        let cluster_id = group.cluster_id();
        let local_id = group.local_id();
        assert_eq!(group.samples.len(), 1);
        assert_eq!(
            group.get_unstored_samples().first().unwrap().sample,
            sample_context
        );
        assert_eq!(parser.samples_counts.get(&cluster_id), Some(&1));

        // Mark all samples as stored
        parser.mark_cluster_samples_as_stored(local_id);

        // This line will be added to samples
        // We are not gonna mark samples as stored to prevent them from being cleared
        let (group, _) = parser.add_log_line("test message 2", None);
        let cluster_id = group.cluster_id();
        assert_eq!(group.samples.len(), 2);
        assert_eq!(parser.samples_counts.get(&cluster_id), Some(&2));

        // At this point the threshold is reached
        let (group, _) = parser.add_log_line("test message 3", None);
        let cluster_id = group.cluster_id();
        let local_id = group.local_id();

        // Samples map should NOT be cleared yet cause there are unstored samples
        assert_eq!(group.samples.len(), 2);
        assert_eq!(parser.samples_counts.get(&cluster_id), Some(&2));
        // Mark all samples as stored
        parser.mark_cluster_samples_as_stored(local_id);

        let (group, _) = parser.add_log_line("test message 4", None);

        // Samples map SHOULD be cleared
        assert_eq!(group.samples.len(), 0);
        assert_eq!(parser.samples_counts.get(&cluster_id), Some(&2));
    }

    #[test]
    fn collect_samples_clusters_overflow() {
        let lines = vec![
            "[LocalLog partition=__cluster_metadata-0, dir=/tmp/kafka-logs] Deleting segment files LogSegment(baseOffset=3094, size=2160, lastModifiedTime=1727197040113, largestRecordTimestamp=1727197040088),LogSegment(baseOffset=3124, size=2160, lastModifiedTime=1727197055158, largestRecordTimestamp=1727197055133)",
            "[LocalLog partition=__cluster_metadata-0, dir=/tmp/kafka-logs] Deleting segment files LogSegment(baseOffset=3094, size=2160, lastModifiedTime=1727197040113, largestRecordTimestamp=1727197040088),LogSegment(baseOffset=3124, size=2160, lastModifiedTime=1727197055158, largestRecordTimestamp=1727197055134)",
            "test message 1",
            "test message 1",
            "test message 2",
            "test message 2",
            "test message 2",
            "test message N",
            "[LocalLog partition=__cluster_metadata-0, dir=/tmp/kafka-logs] Deleting segment files LogSegment(baseOffset=3094, size=2160, lastModifiedTime=1727197040113, largestRecordTimestamp=1727197040088),LogSegment(baseOffset=3124, size=2160, lastModifiedTime=1727197055158, largestRecordTimestamp=1727197055135)",
            "[LocalLog partition=__cluster_metadata-0, dir=/tmp/kafka-logs] Deleting segment files LogSegment(baseOffset=3094, size=2160, lastModifiedTime=1727197040113, largestRecordTimestamp=1727197040088),LogSegment(baseOffset=3124, size=2160, lastModifiedTime=1727197055158, largestRecordTimestamp=1727197055136)",
            "Something completely different 1",
            "Something completely different 1",
            "Something completely different 3",
            "Something completely different 3",
            "Something completely different 2",
            "Something completely different 2",
            "Something completely different N",
            "Successfully connected to Mongo",
            "Successfully connected to Mongo",
            "Successfully connected to Mongo",
            "Successfully connected to Redis",
            "Successfully connected to Redis",
            "Successfully connected to Redis",
        ];

        let expected = vec![
            "[LocalLog partition=__cluster_metadata-0, dir=/tmp/kafka-logs] Deleting segment files LogSegment(baseOffset=3094, size=2160, lastModifiedTime=1727197040113, largestRecordTimestamp=1727197040088),LogSegment(baseOffset=3124, size=2160, lastModifiedTime=1727197055158, largestRecordTimestamp=1727197055133)",
            "[LocalLog partition=__cluster_metadata-0, dir=/tmp/kafka-logs] Deleting segment files LogSegment(baseOffset=3094, size=2160, lastModifiedTime=1727197040113, largestRecordTimestamp=1727197040088),LogSegment(baseOffset=3124, size=2160, lastModifiedTime=1727197055158, largestRecordTimestamp=1727197055134)",
            "test message 1",
            "test message 2",
            "Something completely different 1",
            "Something completely different 3",
            "Successfully connected to Mongo",
            "Successfully connected to Redis",
        ];

        let mut parser =
            LogParser::new(NonZeroUsize::new(3).unwrap(), NonZeroUsize::new(2).unwrap());

        let mut stored_samples: Vec<String> = Vec::new();

        for line in lines.iter() {
            let (group, _) = parser.add_log_line(line, None);

            let samples = group.get_unstored_samples();
            samples
                .iter()
                .for_each(|s| stored_samples.push(s.line.clone()));

            let local_id = group.local_id();
            parser.mark_cluster_samples_as_stored(local_id);
        }

        assert_eq!(stored_samples.len(), expected.len());

        for (line, expected) in stored_samples.iter().zip(expected.iter()) {
            assert_eq!(*line, expected.to_string());
        }
    }
}
