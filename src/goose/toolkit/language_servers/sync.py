from goose.language_server.client import LanguageServerClient
from goose.language_server.config import MultilspyConfig
from goose.language_server.implementations.jedi import JediServer
from goose.language_server.logger import MultilspyLogger


sls = LanguageServerClient()
sls.register_language_server(JediServer.from_env(config=MultilspyConfig(), logger=MultilspyLogger()))
with sls.start_servers() as _:
    d = sls.request_definition("/Users/lalvoeiro/Development/goose/src/goose/build.py", 41, 28)
    print(d)