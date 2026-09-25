use apex_kinetic_vision_node::VisionNodeConfig;

#[test]
fn default_config_matches_deployment_assumptions() {
    let config = VisionNodeConfig::from_env();

    assert_eq!(config.source_url, "rtsp://edge-camera.local/stream");
    assert_eq!(config.target_host, "annke-nvr.local");
    assert_eq!(config.target_port, "554");
    assert_eq!(config.cert_path, "/etc/certs/client.crt");
    assert_eq!(config.key_path, "/etc/certs/client.key");
    assert_eq!(config.ca_path, "/etc/certs/ca.crt");
}
