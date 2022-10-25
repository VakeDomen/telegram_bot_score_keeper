#[derive(Debug)]
pub enum TarokGameInput {
    TarokGame(TarokGame),
    TarokGameAttribute(TarokGameAttribute),
    TarokGameDiff(i32),
}

#[derive(Debug)]
pub enum TarokPlayerInput {
    PlayerAttribute(TarokPlayerAttibute),
    PlayerDiff(i32),
}

#[derive(Debug, Clone, Copy)]
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
    ZP,
    ZK,
    V,
    T,
    K,
    NZP,
    NZK,
    NV,
    NT,
    NK,
}

#[derive(Debug)]
pub enum TarokPlayerAttibute {
    M,
    R,
    T,
    Ig,
    Sl,
}

#[derive(Debug)]
pub enum Radlc {
    Avalible,
    Used,
}