// Beacon API Client

/*
    Todo: Add a client to get committee updates and finality updates
    Submit s

*/

mod types;

/*
This should be a committee update
curl -X 'GET' 'http://65.108.66.225:9596/eth/v1/beacon/light_client/updates?start_period=1316&count=1' -H 'accept: application/json'


This should be a "sync step"
curl -X 'GET' 'http://65.108.66.225:9596/eth/v1/beacon/light_client/finality_update' -H 'accept: application/json'



curl -X 'GET' 'http://65.108.66.225:9596/eth/v1/beacon/states/head/finality_checkpoints -H 'accept: application/json'
*/

#[cfg(test)]
mod tests {
    #[tokio::test]
    pub async fn get_committee_update() {
        let response = reqwest::get("http://65.108.66.225:9596/eth/v1/beacon/light_client/updates?start_period=1316&count=1").await.unwrap();
        if response.status().is_success() {
            let body = response.text().await.unwrap();
            println!("Response: {}", &body);
        } else {
            panic!("Request failed with status: {}", response.status());
        }
    }

    #[tokio::test]
    pub async fn get_step_update() {
        let response =
            reqwest::get("http://65.108.66.225:9596/eth/v1/beacon/light_client/finality_update")
                .await
                .unwrap();
        if response.status().is_success() {
            let body = response.text().await.unwrap();
            println!("Response: {}", &body);
        } else {
            panic!("Request failed with status: {}", response.status());
        }
    }
}
