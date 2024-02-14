CREATE USER "bot-download-proxy";
GRANT EXECUTE ON FUNCTION download_proxy_file_info TO "bot-download-proxy";
GRANT INSERT ON download_proxy_access_log TO "bot-download-proxy";
