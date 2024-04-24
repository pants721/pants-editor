
#[derive(Default)]
pub struct Search {
    pub query: String,
    pub results: Vec<SearchResult>,
}

#[allow(dead_code)]
pub struct SearchResult {
    pub line: String,
    pub row: usize,
    pub start: usize,
    pub end: usize,
}

impl Search {
    pub fn search(&mut self, lines: &[String]) {
        self.results.clear();
        for (i, line) in lines.iter().enumerate() {
            if let Some(start) = line.find(&self.query) {
                self.results.push(SearchResult {
                    line: line.to_string(),
                    row: i,
                    start,
                    end: start + self.query.len(),
                });
            }
        }
    }
}
