CREATE USER "bot-download-proxy";
ALTER USER "bot-download-proxy" SET search_path TO data_gateways;
GRANT EXECUTE ON FUNCTION download_proxy_file_info TO "bot-download-proxy";
GRANT INSERT ON download_proxy_access_log TO "bot-download-proxy";
