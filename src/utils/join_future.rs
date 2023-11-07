use crate::Result;
use std::future::Future;

pub async fn join_all<T: Send + 'static, F: Future<Output = Result<T>> + Send + 'static>(
    futures: impl IntoIterator<Item = F>,
) -> Result<Vec<T>> {
    let mut targets = vec![];
    let futures = futures
        .into_iter()
        .map(|f| tokio::spawn(f))
        .collect::<Vec<_>>();
    for future in futures.into_iter() {
        let t = future.await.unwrap()?;
        targets.push(t);
    }
    Ok(targets)
}
