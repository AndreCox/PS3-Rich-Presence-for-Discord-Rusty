use reqwest::blocking::Client;

pub struct ArtifactUploader {
    pub artifact_url: String,
    webhook_url: String,
    last_artifact_id: String,
}

impl ArtifactUploader {
    pub fn new(webhook_url: String) -> Self {
        Self {
            artifact_url: "".to_string(),
            webhook_url,
            last_artifact_id: "".to_string(),
        }
    }

    pub fn upload_artifact(
        &mut self,
        image_location: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // we'll check if there's already an artifact uploaded
        // if there is, we'll delete it
        if self.last_artifact_id != "".to_string() {
            self.remove_artifact();
        }

        // we create a new client to send our request
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()?;

        // create multipart form data
        let form = reqwest::blocking::multipart::Form::new().file("file", image_location)?;

        // send the request
        let resp = client
            .post(&self.webhook_url)
            .multipart(form)
            .send()?
            .text()?;

        // the response is a json string, so we need to parse it
        let json: serde_json::Value = serde_json::from_str(&resp)?;

        // check if the request was successful by seeing if the id field exists
        if json["id"].is_null() {
            return Err("Failed to upload artifact".into());
        }

        // set the last artifact id to the id of the artifact we just uploaded
        self.last_artifact_id = json["id"].to_string();

        // return the url of the artifact we just uploaded this is stored in attachments[0].url
        self.artifact_url = json["attachments"][0]["url"].to_string();

        Ok(json["attachments"][0]["proxy_url"].to_string())
    }

    fn remove_artifact(&mut self) {
        // we create a new client to send our request
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .unwrap();

        // construct the url to delete the artifact
        let url = format!("{}/messages/{}", self.webhook_url, self.last_artifact_id);

        // send the request
        let _ = client.delete(&url).send();
    }
}
