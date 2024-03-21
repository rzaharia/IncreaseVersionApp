#![allow(unused)]
use anyhow::Result;

#[tokio::test]
async fn quic_dev() -> Result<()> {
    let hc = httpc_test::new_client("http://localhost:3000")?;
    hc.do_get("/callback").await?.print().await?;

    Ok(())
}
