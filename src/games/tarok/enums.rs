#[derive(Debug)]
pub enum TarokGameInput {
    TarokGame(TarokGame),
    TarokGameAttribute(TarokGameAttribute),
    TarokGameDiff(i32),
}

#[derive(Debug)]
pub enum TarokGame {
    I3,
    I2,
    I1,
    S3,
    S2,
    S1,
    SB,
    KL,
    B,
    P,
    BVI3,
    BVI2,
    BVI1,
    BVS3,
    BVS2,
    BVS1,
    BVSB,
}

#[derive(Debug)]
pub enum TarokGameAttribute {
    P,
    K,
    V,
    T,
    NP,
    NK,
    NV,
    NT,
}

#[derive(Debug)]
pub enum TarokPlayerAttibute {
    M,
}