/// A single job and relevant information
#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct MatrixElement {
    finished_at: Option<String>,
    result: Option<u32>,
    number: String,
    id: Option<u32>,
}

/// A list of jobs
#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct Matrix {
    id: u32,
    matrix: Vec<MatrixElement>,
}

impl MatrixElement {
    /// Get the id of the job
    pub fn id(&self) -> u32 {
        self.id.unwrap()
    }

    /// Check if the job was run on the build leader
    pub fn is_leader(&self) -> bool {
        super::is_leader(&self.number)
    }

    /// Check if the job finished and succeeded
    pub fn is_succeeded(&self) -> bool {
        if !self.is_finished() {
           return false;
        }

        match self.result {
            None => false,
            Some(0) => true,
            Some(_) => false,
        }
    }

    /// Check if the job finished
    pub fn is_finished(&self) -> bool {
        self.finished_at.is_some()
    }
}

impl Matrix {
    /// Get the build matrix
    pub fn builds(&self) -> &[MatrixElement] {
        &self.matrix
    }

    /// Check that all non-leader jobs finished
    pub fn others_finished(&self) -> bool {
        self.matrix.iter()
            .filter(|build| !build.is_leader())
            .all(|build| build.is_finished())
    }

    /// Check that all non-leader jobs succeeded
    pub fn others_succeeded(&self) -> bool {
        self.matrix.iter()
            .filter(|build| !build.is_leader())
            .all(|build| build.is_succeeded())
    }
}
