// Beacon API Client

/*
    Todo: Add a client to get committee updates and finality updates
    Submit s

*/

/*
This should be a committee update
curl -X 'GET' 'http://65.108.66.225:9596/eth/v1/beacon/light_client/updates?start_period=1316&count=1' -H 'accept: application/json'


This should be a "sync step"
curl -X 'GET' 'http://65.108.66.225:9596/eth/v1/beacon/light_client/finality_update' -H 'accept: application/json'



curl -X 'GET' 'http://65.108.66.225:9596/eth/v1/beacon/states/head/finality_checkpoints -H 'accept: application/json'
*/

const COMMITTEE_UPDATE_BASE: &str = "http://65.108.66.225:9596/eth/v1/beacon/light_client/updates";
const STEP_UPDATE_BASE: &str =
    "http://65.108.66.225:9596/eth/v1/beacon/light_client/finality_update";
