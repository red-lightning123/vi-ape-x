pub enum Status {
    Running,
    Done(Done),
}

pub enum Done {
    // the naming discrepancy between Terminated and ShouldTruncate is
    // intended. it highlights a semantic difference: termination is
    // performed automatically, while truncation is the caller's
    // reponsibility. this may seem strange, but it follows from the
    // fact that termination is triggered by the game (upon death) and
    // truncation is triggered externally (via truncation_timer)
    //
    // in practice, this means that the caller has to reset the episode
    // on truncation, but not on termination
    Terminated,
    ShouldTruncate,
}
