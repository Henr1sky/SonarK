# Catálogo de ações

Na versão **0.0.10**, o catálogo contém **233 ações executáveis**, incluindo **20** com consulta direta a log HTTP. O inventário tem **159 IDs** ainda sem preset executável.

Na interface, abra a categoria e use **/** para buscar pelo ID. **h** explica a ação e **i** lista os IDs pendentes daquela categoria. Os filtros abaixo são os padrões do catálogo; o SonarK acrescenta os filtros configurados pelo operador.

| Categoria | Ações executáveis | IDs pendentes |
| --- | ---: | ---: |
| [1 · Reconhecimento & Inventário](#categoria-1) | 19 | 6 |
| [2 · Identificação de Atores](#categoria-2) | 16 | 7 |
| [3 · Portas & Serviços](#categoria-3) | 17 | 6 |
| [4 · Payload & Conteúdo](#categoria-4) | 14 | 9 |
| [5 · HTTP / HTTPS](#categoria-5) | 24 | 8 |
| [6 · FTP / Autenticação](#categoria-6) | 12 | 4 |
| [7 · E-mail (SMTP/POP/IMAP)](#categoria-7) | 11 | 4 |
| [8 · Transferência de Arquivos](#categoria-8) | 8 | 11 |
| [9 · SMB / NetBIOS](#categoria-9) | 8 | 7 |
| [10 · DNS](#categoria-10) | 13 | 4 |
| [11 · TLS / SSL](#categoria-11) | 9 | 9 |
| [12 · ARP / ICMP / Camada 2](#categoria-12) | 14 | 1 |
| [13 · Streaming / VoIP](#categoria-13) | 6 | 4 |
| [14 · Banco de Dados](#categoria-14) | 7 | 5 |
| [15 · Acesso Remoto](#categoria-15) | 7 | 5 |
| [16 · Detecção de Anomalias](#categoria-16) | 13 | 5 |
| [17 · Forense Avançada](#categoria-17) | 9 | 6 |
| [18 · Documentos (PDF/Office/Imagens)](#categoria-18) | 3 | 17 |
| [19 · Conversões & Extração](#categoria-19) | 10 | 12 |
| [20 · Interação com Serviços](#categoria-20) | 0 | 14 |
| [21 · Relatório & Exportação](#categoria-21) | 5 | 12 |
| [22 · Aprender & Tutor](#categoria-22) | 0 | 0 |
| [23 · Workspace & Projetos](#categoria-23) | 0 | 0 |
| [24 · BGP / Roteamento](#categoria-24) | 1 | 0 |
| [25 · SNMP](#categoria-25) | 1 | 0 |
| [26 · Industrial / SCADA / ICS](#categoria-26) | 1 | 0 |
| [27 · Bluetooth / BLE](#categoria-27) | 1 | 0 |
| [34 · Análise de Identidade & SSO](#categoria-34) | 4 | 3 |

## Categoria 1

**Reconhecimento & Inventário**

| ID | Ação | Entrada | Operação padrão |
| --- | --- | --- | --- |
| 1.1 | Hierarquia de protocolos | PCAP | Estatísticas TShark `io,phs`. |
| 1.2 | Endpoints (IPv4, IPv6, TCP, UDP, Ethernet) | PCAP | Estatísticas TShark `endpoints,ip`, `endpoints,ipv6`, `endpoints,tcp`, `endpoints,udp`, `endpoints,eth`. |
| 1.3 | Conversas (IPv4, IPv6, TCP, UDP, Ethernet) | PCAP | Estatísticas TShark `conv,ip`, `conv,ipv6`, `conv,tcp`, `conv,udp`, `conv,eth`. |
| 1.4 | Resumo da captura | PCAP / log HTTP | Metadados com `capinfos`. Em log HTTP, usa a consulta nativa. |
| 1.5 | Frequência de MACs de origem | PCAP | Filtro `eth.src`. Campos `eth.src`. Frequência. |
| 1.6 | Evidências de hostname DHCP, MAC e IP | PCAP | Filtro `dhcp.option.hostname`. Campos `eth.src`, `dhcp.ip.client`, `dhcp.ip.your`, `dhcp.option.hostname`. Linhas únicas. |
| 1.7 | Pacotes e bytes por camada de protocolo | PCAP / log HTTP | Estatísticas TShark `io,phs`. Em log HTTP, usa a consulta nativa. |
| 1.9 | Timeline de tráfego (1, 10 e 60 segundos) | PCAP / log HTTP | Estatísticas TShark `io,stat,1`, `io,stat,10`, `io,stat,60`. Em log HTTP, usa a consulta nativa. |
| 1.10 | Expert Info do Wireshark | PCAP | Estatísticas TShark `expert`. |
| 1.11 | Hierarquia de protocolos por bytes | PCAP | Estatísticas TShark `io,phs`. |
| 1.12 | Endpoints por volume de tráfego | PCAP / log HTTP | Estatísticas TShark `endpoints,ip`, `endpoints,ipv6`. Em log HTTP, usa a consulta nativa. |
| 1.13 | Distribuição de tamanhos de quadros | PCAP | Campos `frame.len`. Frequência. |
| 1.14 | Encapsulamento (VLAN, GRE, VXLAN, MPLS) | PCAP | Filtro `vlan \|\| gre \|\| vxlan \|\| mpls`. Campos `frame.number`, `frame.protocols`, `vlan.id`, `vxlan.vni`, `mpls.label`. |
| 1.15 | Janela temporal da captura | PCAP / log HTTP | Metadados com `capinfos` (`-a`, `-e`). Em log HTTP, usa a consulta nativa. |
| 1.16 | Distribuição de TTL IPv4 | PCAP | Filtro `ip.ttl`. Campos `ip.ttl`. Frequência. |
| 1.17 | Fragmentos IPv4 e quadros grandes | PCAP | Filtro `ip.flags.mf == 1 \|\| ip.frag_offset > 0 \|\| frame.len > 1518`. Campos `frame.number`, `ip.src`, `ip.dst`, `frame.len`, `ip.frag_offset`. |
| 1.18 | Contagem Ethernet unicast, multicast e broadcast | PCAP | Filtro `eth.dst`. Campos `eth.dst`. Classificação de endereços MAC. |
| 1.20 | Quadros sem dissecação ou malformados | PCAP | Filtro `data \|\| _ws.malformed`. Campos `frame.number`, `frame.protocols`, `_ws.expert.message`. |
| 1.24 | Eventos de retransmissão TCP e ACK duplicado | PCAP | Filtro `tcp.analysis.retransmission \|\| tcp.analysis.fast_retransmission \|\| tcp.analysis.spurious_retransmission \|\| tcp.analysis.duplicate_ack`. Campos `frame.number`, `tcp.stream`, `ip.src`, `ipv6.src`, `_ws.expert.message`. |

## Categoria 2

**Identificação de Atores**

| ID | Ação | Entrada | Operação padrão |
| --- | --- | --- | --- |
| 2.1 | Iniciadores TCP observados (SYN) | PCAP | Filtro `tcp.flags.syn == 1 && tcp.flags.ack == 0`. Campos `ip.src`, `ipv6.src`, `tcp.dstport`. Frequência. |
| 2.2 | Respondentes TCP observados (SYN+ACK) | PCAP | Filtro `tcp.flags.syn == 1 && tcp.flags.ack == 1`. Campos `ip.src`, `ipv6.src`. Frequência. Editor completo disponível. |
| 2.3 | Pares de IP e MAC de origem | PCAP | Filtro `eth.src && (ip \|\| ipv6)`. Campos `eth.src`, `ip.src`, `ipv6.src`. Linhas únicas. |
| 2.4 | Nomes observados em DHCP e DNS | PCAP | Filtro `dhcp.option.hostname \|\| dns.flags.response == 1 \|\| nbns.name`. Campos `frame.number`, `dhcp.option.hostname`, `dns.qry.name`, `dns.a`, `dns.aaaa`, `nbns.name`. Linhas únicas. |
| 2.5 | Frequência de portas TCP de origem por origem | PCAP | Filtro `tcp`. Campos `ip.src`, `ipv6.src`, `tcp.srcport`. Frequência. |
| 2.7 | Distribuição de flags TCP | PCAP | Filtro `tcp`. Campos `tcp.flags.str`. Frequência. |
| 2.9 | Pares de IPs observados | PCAP | Filtro `ip \|\| ipv6`. Campos `ip.src`, `ipv6.src`, `ip.dst`, `ipv6.dst`. Linhas únicas. |
| 2.10 | Destinos IPv4 fora das faixas RFC 1918 | PCAP | Filtro `ip.dst && !(ip.dst in {10.0.0.0/8,172.16.0.0/12,192.168.0.0/16})`. Campos `ip.dst`. Linhas únicas. |
| 2.11 | Frequência de portas de destino por host | PCAP | Filtro `tcp \|\| udp`. Campos `ip.dst`, `ipv6.dst`, `tcp.dstport`, `udp.dstport`. Frequência. |
| 2.12 | Portas da faixa dinâmica em pacotes TCP SYN | PCAP | Filtro `tcp.srcport >= 49152 && tcp.flags.syn == 1 && tcp.flags.ack == 0`. Campos `ip.src`, `ipv6.src`, `tcp.srcport`. Frequência. |
| 2.13 | Pares de comunicação por frequência | PCAP | Filtro `ip \|\| ipv6`. Campos `ip.src`, `ipv6.src`, `ip.dst`, `ipv6.dst`. Frequência. |
| 2.14 | Conversas IP para investigar pivôs | PCAP | Filtro `ip \|\| ipv6`. Campos `frame.time_epoch`, `ip.src`, `ipv6.src`, `ip.dst`, `ipv6.dst`. |
| 2.15 | Valores HTTP X-Forwarded-For observados | PCAP | Filtro `http.x_forwarded_for`. Campos `ip.src`, `ipv6.src`, `http.host`, `http.x_forwarded_for`. Linhas únicas. |
| 2.19 | Observações de MAC e IP por VLAN | PCAP | Filtro `vlan && eth.src`. Campos `vlan.id`, `eth.src`, `ip.src`, `ipv6.src`. Linhas únicas. |
| 2.20 | Características passivas de TCP SYN | PCAP | Filtro `tcp.flags.syn == 1 && tcp.flags.ack == 0`. Campos `ip.src`, `ipv6.src`, `ip.ttl`, `ipv6.hlim`, `tcp.window_size_value`, `tcp.options.mss_val`. Linhas únicas. |
| 2.21 | Múltiplos MACs Ethernet de origem observados para um IPv4 | PCAP | Filtro `ip.src && eth.src`. Campos `ip.src`, `eth.src`. Chaves com valores distintos. |

## Categoria 3

**Portas & Serviços**

| ID | Ação | Entrada | Operação padrão |
| --- | --- | --- | --- |
| 3.1 | Portas TCP observadas enviando SYN+ACK | PCAP | Filtro `tcp.flags.syn == 1 && tcp.flags.ack == 1`. Campos `ip.src`, `ipv6.src`, `tcp.srcport`. Linhas únicas. |
| 3.2 | Portas TCP observadas enviando RST | PCAP | Filtro `tcp.flags.reset == 1`. Campos `ip.src`, `ipv6.src`, `tcp.srcport`. Linhas únicas. |
| 3.4 | Fluxos com payload TCP | PCAP | Filtro `tcp.len > 0`. Campos `ip.src`, `ipv6.src`, `tcp.srcport`, `ip.dst`, `ipv6.dst`, `tcp.dstport`. Frequência. |
| 3.5 | Portas enviando TCP FIN | PCAP | Filtro `tcp.flags.fin == 1`. Campos `ip.src`, `ipv6.src`, `tcp.srcport`. Frequência. |
| 3.6 | Portas de origem UDP observadas | PCAP | Filtro `udp`. Campos `ip.src`, `ipv6.src`, `udp.srcport`. Linhas únicas. |
| 3.7 | Frequência de portas de destino de SYN | PCAP | Filtro `tcp.flags.syn == 1 && tcp.flags.ack == 0`. Campos `tcp.dstport`. Frequência. Editor completo disponível. |
| 3.8 | Amostras de payload TCP por porta de origem | PCAP | Filtro `tcp.len > 0`. Campos `tcp.srcport`, `tcp.payload`. |
| 3.9 | Portas e protocolos dissecados | PCAP | Filtro `tcp \|\| udp`. Campos `tcp.dstport`, `udp.dstport`, `frame.protocols`. Linhas únicas. |
| 3.10 | Tentativas TCP SYN com horário | PCAP | Filtro `tcp.flags.syn == 1 && tcp.flags.ack == 0`. Campos `frame.time_relative`, `ip.src`, `ipv6.src`, `ip.dst`, `ipv6.dst`, `tcp.dstport`. |
| 3.11 | Portas TCP de destino por host | PCAP | Filtro `tcp`. Campos `ip.dst`, `ipv6.dst`, `tcp.dstport`. Frequência. |
| 3.12 | Origens e destinos de RST | PCAP | Filtro `tcp.flags.reset == 1`. Campos `ip.src`, `ipv6.src`, `tcp.srcport`, `ip.dst`, `ipv6.dst`, `tcp.dstport`. Frequência. |
| 3.13 | Payload observado em portas TCP altas | PCAP | Filtro `tcp.dstport >= 1024 && tcp.len > 0`. Campos `tcp.dstport`, `frame.protocols`, `tcp.payload`. |
| 3.14 | Destinos UDP observados por origem | PCAP | Filtro `udp`. Campos `ip.src`, `ipv6.src`, `ip.dst`, `ipv6.dst`, `udp.dstport`. Frequência. |
| 3.15 | Indícios de TCP XMAS, NULL e FIN | PCAP | Filtro `tcp.flags == 0x029 \|\| tcp.flags == 0 \|\| tcp.flags == 1`. Campos `frame.number`, `ip.src`, `ipv6.src`, `ip.dst`, `ipv6.dst`, `tcp.dstport`, `tcp.flags`. |
| 3.16 | Horários de tentativas TCP SYN | PCAP | Filtro `tcp.flags.syn == 1 && tcp.flags.ack == 0`. Campos `frame.time_relative`, `ip.src`, `ipv6.src`, `tcp.dstport`. |
| 3.19 | Destinos com horários para analisar port knocking | PCAP | Filtro `tcp.flags.syn == 1 && tcp.flags.ack == 0`. Campos `frame.time_relative`, `ip.src`, `ipv6.src`, `ip.dst`, `ipv6.dst`, `tcp.dstport`. |
| 3.20 | Evidências de sequência TCP SYN, SYN+ACK e RST | PCAP | Filtro `tcp.flags.syn == 1 \|\| tcp.flags.reset == 1`. Campos `frame.number`, `tcp.stream`, `ip.src`, `ipv6.src`, `ip.dst`, `ipv6.dst`, `tcp.flags.str`. |

## Categoria 4

**Payload & Conteúdo**

| ID | Ação | Entrada | Operação padrão |
| --- | --- | --- | --- |
| 4.1 | Payload TCP bruto em hex | PCAP | Filtro `tcp.payload`. Campos `tcp.payload`. |
| 4.2 | Decodificar hex de payload TCP como texto por segmento | PCAP | Lê `tcp.payload` e decodifica os bytes por segmento. |
| 4.3 | Buscar no payload TCP (defina o filtro de texto) | PCAP | Filtro `tcp.payload`. Campos `frame.number`, `ip.src`, `ipv6.src`, `tcp.payload`. |
| 4.4 | Payload TCP e IP de origem | PCAP | Filtro `tcp.payload`. Campos `ip.src`, `ipv6.src`, `tcp.payload`. |
| 4.5 | Payload TCP e pares de sockets | PCAP | Filtro `tcp.payload`. Campos `ip.src`, `ipv6.src`, `tcp.srcport`, `ip.dst`, `ipv6.dst`, `tcp.dstport`, `tcp.payload`. |
| 4.6 | Payload TCP por porta (adicione filtro de porta) | PCAP | Filtro `tcp.payload`. Campos `ip.src`, `ipv6.src`, `tcp.srcport`, `tcp.dstport`, `tcp.payload`. Editor completo disponível. |
| 4.7 | Strings imprimíveis em payload TCP | PCAP | Lê `tcp.payload` e mostra sequências imprimíveis. |
| 4.8 | Seguir stream TCP (ASCII) | PCAP | Segue stream TCP em ASCII. Configure o número em **a → s**. |
| 4.9 | Seguir stream UDP (ASCII) | PCAP | Segue stream UDP em ASCII. Configure o número em **a → s**. |
| 4.10 | Buscar no payload TCP (defina o filtro de assinatura em bytes) | PCAP | Filtro `tcp.payload`. Campos `frame.number`, `tcp.stream`, `ip.src`, `ipv6.src`. |
| 4.11 | Payload TCP por host (adicione filtro de host) | PCAP | Filtro `tcp.payload`. Campos `ip.src`, `ipv6.src`, `ip.dst`, `ipv6.dst`, `tcp.payload`. |
| 4.14 | Tamanho médio de payload TCP positivo por stream | PCAP | Filtro `tcp.len > 0`. Campos `tcp.stream`, `tcp.len`. Média por chave. |
| 4.18 | Buscar no payload TCP (defina o filtro por regex) | PCAP | Filtro `tcp.payload`. Campos `frame.number`, `tcp.stream`, `tcp.payload`. |
| 4.19 | Evidências de retransmissão e segmentos fora de ordem | PCAP | Filtro `tcp.analysis.retransmission \|\| tcp.analysis.out_of_order`. Campos `tcp.stream`, `frame.number`, `tcp.seq`. |

## Categoria 5

**HTTP / HTTPS**

| ID | Ação | Entrada | Operação padrão |
| --- | --- | --- | --- |
| 5.1 | Referer | PCAP / log HTTP | Filtro `http.referer`. Campos `http.referer`. Em log HTTP, usa a consulta nativa. |
| 5.2 | User-Agent | PCAP / log HTTP | Filtro `http.user_agent`. Campos `http.user_agent`. Em log HTTP, usa a consulta nativa. |
| 5.3 | Host | PCAP | Filtro `http.host`. Campos `http.host`. |
| 5.4 | URIs requisitadas | PCAP / log HTTP | Filtro `http.request.uri`. Campos `http.request.uri`. Em log HTTP, usa a consulta nativa. |
| 5.5 | Cabeçalho Server | PCAP | Filtro `http.server`. Campos `http.server`. |
| 5.6 | Métodos HTTP | PCAP / log HTTP | Filtro `http.request.method`. Campos `http.request.method`. Em log HTTP, usa a consulta nativa. |
| 5.7 | Códigos de status HTTP | PCAP / log HTTP | Filtro `http.response.code`. Campos `http.response.code`. Em log HTTP, usa a consulta nativa. |
| 5.8 | Parâmetros de URL | PCAP / log HTTP | Filtro `http.request.uri.query`. Campos `http.request.uri.query`. Em log HTTP, usa a consulta nativa. |
| 5.9 | Cabeçalhos HTTP de requisição | PCAP | Filtro `http.request.line`. Campos `frame.number`, `http.request.line`. |
| 5.10 | Presença de cabeçalhos de cookies HTTP | PCAP | Filtro `http.cookie \|\| http.set_cookie`. Campos `frame.number`, `http.host`, `http.request.uri`. Linhas únicas. |
| 5.12 | Corpos de requisições HTTP POST | PCAP / log HTTP | Filtro `http.request.method == "POST" && http.file_data`. Campos `frame.number`, `http.host`, `http.request.uri`, `http.file_data`. Em log HTTP, usa a consulta nativa. |
| 5.13 | Exportar objetos HTTP | PCAP | Escreve arquivos em uma pasta de exportação com o dissector `http`. |
| 5.14 | Redirecionamentos HTTP | PCAP / log HTTP | Filtro `http.response.code >= 300 && http.response.code < 400`. Campos `frame.number`, `http.response.code`, `http.location`. Em log HTTP, usa a consulta nativa. |
| 5.15 | Requisições HTTP pelo cabeçalho Host | PCAP | Filtro `http.request && http.host`. Campos `http.host`. Frequência. |
| 5.16 | Timeline de requisições HTTP | PCAP / log HTTP | Filtro `http.request`. Campos `frame.time_epoch`, `http.request.method`, `http.host`, `http.request.uri`. Em log HTTP, usa a consulta nativa. |
| 5.17 | Uploads HTTP multipart | PCAP / log HTTP | Filtro `http.request.method == "POST" && http.content_type contains "multipart/form-data"`. Campos `frame.number`, `http.host`, `http.request.uri`, `http.content_type`. Em log HTTP, usa a consulta nativa. |
| 5.18 | Content-Disposition em HTTP | PCAP | Filtro `http.response.line matches "(?i)^content-disposition:"`. Campos `frame.number`, `http.response.line`. |
| 5.19 | Quadros HTTP/2 e HTTP/3 | PCAP | Filtro `http2 \|\| http3`. Campos `frame.number`, `frame.protocols`. |
| 5.20 | Requisições HTTP CONNECT de proxy | PCAP / log HTTP | Filtro `http.request.method == "CONNECT"`. Campos `ip.src`, `ipv6.src`, `http.host`, `http.request.uri`. Em log HTTP, usa a consulta nativa. |
| 5.21 | Upgrades HTTP para WebSocket | PCAP | Filtro `http.upgrade contains "websocket"`. Campos `frame.number`, `http.host`, `http.request.uri`, `http.upgrade`. |
| 5.22 | Metadados de conteúdo JSON via HTTP | PCAP | Filtro `http.content_type contains "application/json"`. Campos `frame.number`, `http.host`, `http.request.uri`, `http.content_length_header`. |
| 5.23 | Cabeçalhos HTTP de CORS | PCAP | Filtro `http.response.line matches "(?i)^access-control-allow-origin:"`. Campos `frame.number`, `http.response.line`. |
| 5.24 | Cabeçalhos HTTP de cache | PCAP | Filtro `http.cache_control`. Campos `frame.number`, `http.cache_control`. |
| 5.32 | Múltiplas requisições HTTP dissecadas no mesmo quadro | PCAP | Filtro `count(http.request.method) > 1`. Campos `frame.number`, `tcp.stream`, `http.request.method`, `http.request.uri`. |

## Categoria 6

**FTP / Autenticação**

| ID | Ação | Entrada | Operação padrão |
| --- | --- | --- | --- |
| 6.1 | Banner do servidor FTP | PCAP | Filtro `ftp.response.code == 220`. Campos `ip.src`, `ipv6.src`, `ftp.response.arg`. |
| 6.3 | Frequência de comandos FTP | PCAP | Filtro `ftp.request.command`. Campos `ftp.request.command`. Frequência. |
| 6.4 | Códigos de resposta FTP | PCAP | Filtro `ftp.response.code`. Campos `ftp.response.code`. Frequência. |
| 6.6 | Transações FTP em modo passivo | PCAP | Filtro `ftp.request.command == "PASV" \|\| ftp.response.code == 227`. Campos `frame.number`, `tcp.stream`, `ftp.request.command`, `ftp.response.arg`. |
| 6.7 | Endereço e porta do modo ativo FTP | PCAP | Filtro `ftp.request.command == "PORT"`. Campos `frame.number`, `ip.src`, `ipv6.src`, `ftp.request.arg`. |
| 6.8 | Solicitações de transferência FTP | PCAP | Filtro `ftp.request.command == "RETR" \|\| ftp.request.command == "STOR"`. Campos `frame.number`, `ftp.request.command`, `ftp.request.arg`. |
| 6.9 | Exportar objetos de dados FTP | PCAP | Escreve arquivos em uma pasta de exportação com o dissector `ftp-data`. |
| 6.10 | Falhas de login FTP | PCAP | Filtro `ftp.response.code == 530`. Campos `ip.src`, `ipv6.src`, `ip.dst`, `ipv6.dst`. Frequência. |
| 6.12 | Tentativas de login FTP anônimo | PCAP | Filtro `ftp.request.command == "USER" && ftp.request.arg == "anonymous"`. Campos `frame.number`, `ip.src`, `ipv6.src`, `ip.dst`, `ipv6.dst`. |
| 6.13 | Handshakes TLS em portas FTP | PCAP | Filtro `tls.handshake && (tcp.port == 21 \|\| tcp.port == 990)`. Campos `frame.number`, `ip.src`, `ipv6.src`, `ip.dst`, `ipv6.dst`. |
| 6.15 | Timeline de transferências FTP por stream | PCAP | Filtro `ftp.request.command == "RETR" \|\| ftp.request.command == "STOR"`. Campos `frame.time_epoch`, `tcp.stream`, `ftp.request.command`, `ftp.request.arg`. |
| 6.16 | Requisições FTP PORT para investigar bounce | PCAP | Filtro `ftp.request.command == "PORT"`. Campos `ip.src`, `ipv6.src`, `ip.dst`, `ipv6.dst`, `ftp.request.arg`. Frequência. |

## Categoria 7

**E-mail (SMTP/POP/IMAP)**

| ID | Ação | Entrada | Operação padrão |
| --- | --- | --- | --- |
| 7.1 | Comandos de envelope SMTP | PCAP | Filtro `smtp.req.command == "MAIL" \|\| smtp.req.command == "RCPT"`. Campos `frame.number`, `smtp.req.command`, `smtp.req.parameter`. |
| 7.2 | Assuntos de e-mail registrados | PCAP | Filtro `imf.subject`. Campos `frame.number`, `imf.subject`. |
| 7.4 | Metadados de anexos MIME | PCAP | Filtro `mime_multipart.header.content-disposition`. Campos `frame.number`, `mime_multipart.header.content-disposition`. |
| 7.5 | Endpoints SMTP | PCAP | Filtro `smtp`. Campos `ip.src`, `ipv6.src`, `ip.dst`, `ipv6.dst`. Linhas únicas. |
| 7.6 | Nomes de comandos POP | PCAP | Filtro `pop.request.command`. Campos `frame.number`, `pop.request.command`. Frequência. |
| 7.7 | Nomes de comandos IMAP | PCAP | Filtro `imap.request.command`. Campos `frame.number`, `imap.request.command`. Frequência. |
| 7.8 | Exportar objetos de mensagens IMF | PCAP | Escreve arquivos em uma pasta de exportação com o dissector `imf`. |
| 7.9 | TLS em portas SMTP | PCAP | Filtro `tls.handshake && (tcp.port == 465 \|\| tcp.port == 587 \|\| tcp.port == 25)`. Campos `frame.number`, `ip.src`, `ipv6.src`, `tcp.srcport`, `tcp.dstport`. |
| 7.10 | Eventos SMTP AUTH | PCAP | Filtro `smtp.req.command == "AUTH"`. Campos `frame.number`, `ip.src`, `ipv6.src`, `ip.dst`, `ipv6.dst`. |
| 7.13 | Cabeçalhos From e Return-Path de mensagens registradas | PCAP | Filtro `imf.from \|\| imf.return_path`. Campos `frame.number`, `imf.from`, `imf.return_path`. Linhas únicas. |
| 7.14 | Frequência de envelopes SMTP por origem | PCAP | Filtro `smtp.req.command == "MAIL"`. Campos `ip.src`, `ipv6.src`. Frequência. |

## Categoria 8

**Transferência de Arquivos**

| ID | Ação | Entrada | Operação padrão |
| --- | --- | --- | --- |
| 8.1 | Exportar objetos FTP | PCAP | Escreve arquivos em uma pasta de exportação com o dissector `ftp-data`. |
| 8.2 | Exportar objetos HTTP | PCAP | Escreve arquivos em uma pasta de exportação com o dissector `http`. |
| 8.3 | Exportar objetos SMB | PCAP | Escreve arquivos em uma pasta de exportação com o dissector `smb`. |
| 8.4 | Exportar objetos TFTP | PCAP | Escreve arquivos em uma pasta de exportação com o dissector `tftp`. |
| 8.9 | Exportar objetos DICOM | PCAP | Escreve arquivos em uma pasta de exportação com o dissector `dicom`. |
| 8.13 | Transferências HTTP em chunks | PCAP | Filtro `http.transfer_encoding contains "chunked"`. Campos `frame.number`, `http.transfer_encoding`. |
| 8.14 | Assinaturas ZIP, RAR e 7z em payload TCP | PCAP | Filtro `tcp.payload contains 50:4b:03:04 \|\| tcp.payload contains 52:61:72:21 \|\| tcp.payload contains 37:7a:bc:af:27:1c`. Campos `frame.number`, `tcp.stream`, `ip.src`, `ipv6.src`. |
| 8.16 | Bytes de payload TCP capturados por destino IPv4 | PCAP | Filtro `ip.dst && tcp.len > 0`. Campos `ip.dst`, `tcp.len`. Soma por chave. |

## Categoria 9

**SMB / NetBIOS**

| ID | Ação | Entrada | Operação padrão |
| --- | --- | --- | --- |
| 9.1 | Pares de endpoints SMB | PCAP | Filtro `smb \|\| smb2`. Campos `ip.src`, `ipv6.src`, `ip.dst`, `ipv6.dst`. Linhas únicas. |
| 9.2 | Caminhos de compartilhamento SMB2 | PCAP | Filtro `smb2.tree`. Campos `frame.number`, `smb2.tree`. Linhas únicas. |
| 9.3 | Nomes de arquivos SMB2 | PCAP | Filtro `smb2.filename`. Campos `frame.number`, `smb2.filename`. |
| 9.4 | Eventos de autenticação NTLM | PCAP | Filtro `ntlmssp.messagetype == 3`. Campos `frame.number`, `ip.src`, `ipv6.src`, `ntlmssp.auth.domain`, `ntlmssp.auth.username`. |
| 9.5 | Exportar objetos SMB | PCAP | Escreve arquivos em uma pasta de exportação com o dissector `smb`. |
| 9.6 | Versões dos protocolos SMB e SMB2 | PCAP | Filtro `smb \|\| smb2`. Campos `frame.protocols`, `smb2.dialect`. Frequência. |
| 9.7 | Observações da flag de assinatura SMB2 | PCAP | Filtro `smb2.flags.signature`. Campos `frame.number`, `smb2.flags.signature`. Frequência. |
| 9.9 | Nomes NetBIOS | PCAP | Filtro `nbns.name`. Campos `ip.src`, `ipv6.src`, `nbns.name`. Linhas únicas. |

## Categoria 10

**DNS**

| ID | Ação | Entrada | Operação padrão |
| --- | --- | --- | --- |
| 10.1 | Consultas DNS | PCAP | Filtro `dns.flags.response == 0 && dns.qry.name`. Campos `frame.number`, `ip.src`, `ipv6.src`, `dns.qry.name`, `dns.qry.type`. |
| 10.2 | Respostas DNS | PCAP | Filtro `dns.flags.response == 1`. Campos `frame.number`, `dns.qry.name`, `dns.flags.rcode`, `dns.a`, `dns.aaaa`, `dns.cname`. |
| 10.3 | NXDOMAIN por consulta | PCAP | Filtro `dns.flags.rcode == 3 && dns.qry.name`. Campos `dns.qry.name`. Frequência. |
| 10.4 | Tipos de registro DNS | PCAP | Filtro `dns.resp.type`. Campos `dns.resp.type`. Frequência. |
| 10.5 | Nomes DNS mais consultados | PCAP | Filtro `dns.flags.response == 0 && dns.qry.name`. Campos `dns.qry.name`. Frequência. |
| 10.6 | Respostas DNS TXT para investigar túnel | PCAP | Filtro `dns.txt`. Campos `frame.number`, `ip.src`, `ipv6.src`, `dns.qry.name`, `dns.txt`. |
| 10.7 | Nomes DNS longos | PCAP | Filtro `dns.qry.name.len > 50`. Campos `frame.number`, `dns.qry.name`. Linhas únicas. |
| 10.8 | Frequência de endereços de servidores DNS | PCAP | Filtro `dns.flags.response == 0`. Campos `ip.dst`, `ipv6.dst`. Frequência. |
| 10.9 | Múltiplas respostas IPv4 de DNS por consulta | PCAP | Filtro `dns.flags.response == 1 && dns.qry.name && dns.a`. Campos `dns.qry.name`, `dns.a`. Chaves com valores distintos. |
| 10.10 | Endereços de respostas DNS e TTLs | PCAP | Filtro `dns.flags.response == 1 && (dns.a \|\| dns.aaaa)`. Campos `dns.qry.name`, `dns.a`, `dns.aaaa`, `dns.resp.ttl`. Linhas únicas. |
| 10.11 | Content-Type HTTP visível de DNS sobre HTTPS | PCAP | Filtro `http.content_type contains "application/dns-message"`. Campos `frame.number`, `http.host`, `http.request.uri`, `http.content_type`. |
| 10.12 | Handshakes TLS na porta de DNS sobre TLS | PCAP | Filtro `tls.handshake && tcp.port == 853`. Campos `frame.number`, `ip.src`, `ipv6.src`, `ip.dst`, `ipv6.dst`. |
| 10.13 | Observações de registros DNSSEC | PCAP | Filtro `dns.resp.type in {43,46,47,48,50,51}`. Campos `frame.number`, `dns.qry.name`, `dns.resp.type`. |

## Categoria 11

**TLS / SSL**

| ID | Ação | Entrada | Operação padrão |
| --- | --- | --- | --- |
| 11.1 | Nomes de servidor TLS (SNI) | PCAP | Filtro `tls.handshake.extensions_server_name`. Campos `frame.number`, `tls.handshake.extensions_server_name`. |
| 11.2 | Nomes e campos de validade de certificados TLS | PCAP | Filtro `tls.handshake.type == 11`. Campos `frame.number`, `x509sat.uTF8String`, `x509ce.dNSName`, `x509af.utcTime`. |
| 11.3 | Campos de versão do TLS ServerHello | PCAP | Filtro `tls.handshake.type == 2`. Campos `tls.handshake.version`, `tls.handshake.extensions.supported_version`. Frequência. |
| 11.4 | Cipher suite do TLS ServerHello | PCAP | Filtro `tls.handshake.type == 2`. Campos `frame.number`, `tls.handshake.ciphersuite`. |
| 11.5 | Mensagens de handshake TLS | PCAP | Filtro `tls.handshake.type`. Campos `frame.number`, `tcp.stream`, `tls.handshake.type`. |
| 11.6 | Fingerprints JA3 de clientes | PCAP | Filtro `tls.handshake.ja3`. Campos `tls.handshake.ja3`. Frequência. |
| 11.7 | Fingerprints JA3S de servidores | PCAP | Filtro `tls.handshake.ja3s`. Campos `tls.handshake.ja3s`. Frequência. |
| 11.10 | Algoritmos de assinatura de certificados TLS | PCAP | Filtro `tls.handshake.type == 11`. Campos `frame.number`, `x509af.algorithm.id`. |
| 11.14 | TLS dissecado fora das portas HTTPS usuais | PCAP | Filtro `tls.handshake && !(tcp.port in {443,8443})`. Campos `frame.number`, `ip.src`, `ipv6.src`, `tcp.srcport`, `tcp.dstport`. |

## Categoria 12

**ARP / ICMP / Camada 2**

| ID | Ação | Entrada | Operação padrão |
| --- | --- | --- | --- |
| 12.1 | Requisições e respostas ARP | PCAP | Filtro `arp`. Campos `frame.number`, `arp.opcode`, `arp.src.proto_ipv4`, `arp.src.hw_mac`, `arp.dst.proto_ipv4`. |
| 12.2 | Endereços IPv4 anunciados por múltiplos MACs de origem no ARP | PCAP | Filtro `arp.src.proto_ipv4 && arp.src.hw_mac && arp.src.proto_ipv4 != 0.0.0.0`. Campos `arp.src.proto_ipv4`, `arp.src.hw_mac`. Chaves com valores distintos. |
| 12.3 | ARP gratuito | PCAP | Filtro `arp.isgratuitous`. Campos `frame.number`, `arp.src.proto_ipv4`, `arp.src.hw_mac`. |
| 12.4 | Requisições e respostas ICMP echo | PCAP | Filtro `icmp.type == 8 \|\| icmp.type == 0`. Campos `frame.number`, `ip.src`, `ip.dst`, `icmp.type`, `icmp.seq`. |
| 12.5 | Evidências de payload em requisições ICMP echo | PCAP | Filtro `icmp.type == 8 && data.len > 0`. Campos `frame.number`, `ip.src`, `ip.dst`, `data.len`, `data.data`. |
| 12.6 | Redirecionamentos ICMP | PCAP | Filtro `icmp.type == 5`. Campos `frame.number`, `ip.src`, `ip.dst`, `icmp.redir_gw`. |
| 12.7 | ICMP unreachable e time exceeded | PCAP | Filtro `icmp.type == 3 \|\| icmp.type == 11`. Campos `frame.number`, `ip.src`, `ip.dst`, `icmp.type`, `icmp.code`. |
| 12.8 | Tags VLAN empilhadas | PCAP | Filtro `count(vlan.id) > 1`. Campos `frame.number`, `eth.src`, `eth.dst`, `vlan.id`. |
| 12.9 | Observações de raiz e bridge STP | PCAP | Filtro `stp`. Campos `frame.number`, `stp.root.hw`, `stp.bridge.hw`. |
| 12.10 | Servidores observados em DHCP OFFER e ACK | PCAP | Filtro `dhcp.option.dhcp in {2,5}`. Campos `frame.number`, `ip.src`, `dhcp.option.dhcp_server_id`. |
| 12.11 | Respondentes LLMNR/NBNS | PCAP | Filtro `(llmnr && dns.flags.response == 1) \|\| (nbns && nbns.flags.response == 1)`. Campos `frame.number`, `ip.src`, `ipv6.src`, `dns.qry.name`, `nbns.name`. |
| 12.12 | Anúncios de roteador IPv6 | PCAP | Filtro `icmpv6.type == 134`. Campos `frame.number`, `ipv6.src`, `icmpv6.nd.ra.router_lifetime`. |
| 12.13 | Evidência de endereço ARP duplicado no Wireshark | PCAP | Filtro `arp.duplicate-address-detected`. Campos `frame.number`, `arp.src.hw_mac`, `arp.src.proto_ipv4`. |
| 12.14 | MACs Ethernet de origem únicos por janela de 10 segundos | PCAP | Filtro `eth.src`. Campos `frame.time_relative`, `eth.src`. Valores únicos por janela. |

## Categoria 13

**Streaming / VoIP**

| ID | Ação | Entrada | Operação padrão |
| --- | --- | --- | --- |
| 13.1 | Diálogos SIP | PCAP | Filtro `sip.Call-ID`. Campos `frame.number`, `sip.Call-ID`, `sip.Method`, `sip.Status-Code`. |
| 13.2 | Identificadores de streams RTP | PCAP | Filtro `rtp`. Campos `ip.src`, `ipv6.src`, `udp.srcport`, `ip.dst`, `ipv6.dst`, `udp.dstport`, `rtp.ssrc`. Frequência. |
| 13.4 | Requisições RTSP | PCAP | Filtro `rtsp.method`. Campos `frame.number`, `rtsp.method`, `rtsp.url`. |
| 13.5 | Evidências de endpoints H.323 | PCAP | Filtro `h225 \|\| h245`. Campos `ip.src`, `ipv6.src`, `ip.dst`, `ipv6.dst`. Linhas únicas. |
| 13.8 | Frequência de tipos de payload RTP | PCAP | Filtro `rtp`. Campos `rtp.p_type`. Frequência. |
| 13.10 | Requisições de manifestos HLS e DASH | PCAP | Filtro `http.request.uri contains ".m3u8" \|\| http.request.uri contains ".mpd"`. Campos `frame.number`, `http.host`, `http.request.uri`. |

## Categoria 14

**Banco de Dados**

| ID | Ação | Entrada | Operação padrão |
| --- | --- | --- | --- |
| 14.1 | Consultas MySQL | PCAP | Filtro `mysql.query`. Campos `frame.number`, `mysql.query`. |
| 14.3 | Consultas PostgreSQL | PCAP | Filtro `pgsql.query`. Campos `frame.number`, `pgsql.query`. |
| 14.4 | Texto de consultas MSSQL TDS | PCAP | Filtro `tds.query`. Campos `frame.number`, `tds.query`. |
| 14.5 | Inventário de quadros MongoDB | PCAP | Filtro `mongo`. Campos `frame.number`, `ip.src`, `ipv6.src`, `ip.dst`, `ipv6.dst`. |
| 14.6 | Inventário de quadros Redis | PCAP | Filtro `resp`. Campos `frame.number`, `ip.src`, `ipv6.src`, `ip.dst`, `ipv6.dst`. |
| 14.7 | Endpoints Cassandra | PCAP | Filtro `cql`. Campos `ip.src`, `ipv6.src`, `ip.dst`, `ipv6.dst`. Linhas únicas. |
| 14.8 | Dados de conexão Oracle TNS | PCAP | Filtro `tns.connect_data`. Campos `frame.number`, `tns.connect_data`. |

## Categoria 15

**Acesso Remoto**

| ID | Ação | Entrada | Operação padrão |
| --- | --- | --- | --- |
| 15.1 | Banner de versão SSH | PCAP | Filtro `ssh.protocol`. Campos `frame.number`, `ip.src`, `ipv6.src`, `ssh.protocol`. |
| 15.4 | Endpoints RDP dissecados | PCAP | Filtro `rdp`. Campos `frame.number`, `ip.src`, `ipv6.src`, `ip.dst`, `ipv6.dst`. |
| 15.5 | Tipo de autenticação VNC | PCAP | Filtro `vnc.security_type \|\| vnc.client_security_type`. Campos `frame.number`, `ip.src`, `ipv6.src`, `vnc.security_type`, `vnc.client_security_type`. |
| 15.7 | SSH decodificado fora da porta 22 | PCAP | Filtro `ssh && !(tcp.port == 22)`. Campos `ip.src`, `ipv6.src`, `tcp.srcport`, `ip.dst`, `ipv6.dst`, `tcp.dstport`. Linhas únicas. |
| 15.8 | Handshake TLS na porta RDP | PCAP | Filtro `tls.handshake && tcp.port == 3389`. Campos `frame.number`, `ip.src`, `ipv6.src`, `ip.dst`, `ipv6.dst`. |
| 15.10 | Pares de endpoints de acesso remoto | PCAP | Filtro `ssh \|\| rdp \|\| vnc \|\| telnet`. Campos `ip.src`, `ipv6.src`, `ip.dst`, `ipv6.dst`, `tcp.dstport`, `frame.protocols`. Frequência. |
| 15.12 | Evidências de negociação VNC sem autenticação | PCAP | Filtro `vnc.security_type == 1 \|\| vnc.client_security_type == 1`. Campos `frame.number`, `ip.src`, `ipv6.src`, `ip.dst`, `ipv6.dst`. |

## Categoria 16

**Detecção de Anomalias**

| ID | Ação | Entrada | Operação padrão |
| --- | --- | --- | --- |
| 16.1 | Frequência de TCP SYN por origem e destino | PCAP | Filtro `tcp.flags.syn == 1 && tcp.flags.ack == 0`. Campos `ip.src`, `ipv6.src`, `ip.dst`, `ipv6.dst`, `tcp.dstport`. Frequência. |
| 16.2 | Horários de SYN TCP por destino e porta | PCAP | Filtro `ip.dst && tcp.flags.syn == 1 && tcp.flags.ack == 0`. Campos `ip.dst`, `tcp.dstport`, `frame.time_epoch`. |
| 16.3 | Bytes de payload TCP capturados por destino IPv4 | PCAP | Filtro `ip.dst && tcp.len > 0`. Campos `ip.dst`, `tcp.len`. Soma por chave. |
| 16.6 | Quadros malformados | PCAP / log HTTP | Filtro `_ws.malformed`. Campos `frame.number`, `frame.protocols`, `_ws.expert.message`. Em log HTTP, usa a consulta nativa. |
| 16.7 | Eventos de retransmissão TCP | PCAP | Filtro `tcp.analysis.retransmission \|\| tcp.analysis.fast_retransmission`. Campos `frame.number`, `tcp.stream`, `frame.time_relative`. |
| 16.8 | Expert Info do Wireshark | PCAP | Estatísticas TShark `expert`. |
| 16.9 | Requisições ICMP echo com payload grande | PCAP | Filtro `icmp.type == 8 && data.len > 64`. Campos `frame.number`, `ip.src`, `ip.dst`, `data.len`. |
| 16.10 | Intervalos entre SYNs TCP por destino IPv4 | PCAP | Filtro `ip.dst && tcp.flags.syn == 1 && tcp.flags.ack == 0`. Campos `ip.dst`, `frame.time_epoch`. Intervalos por chave. |
| 16.11 | Observações de deltas de quadros TCP | PCAP | Filtro `tcp`. Campos `frame.number`, `tcp.stream`, `frame.time_delta_displayed`. |
| 16.12 | Segmentos TCP fora de ordem | PCAP | Filtro `tcp.analysis.out_of_order`. Campos `frame.number`, `tcp.stream`, `frame.time_relative`. |
| 16.13 | Tamanhos de quadros observados | PCAP | Campos `frame.len`. Frequência. |
| 16.14 | HTTP ou DNS dissecado em portas incomuns | PCAP | Filtro `(http && tcp && !(tcp.port in {80,443,8080,8443})) \|\| (dns && udp && !(udp.port == 53))`. Campos `frame.number`, `frame.protocols`, `tcp.dstport`, `udp.dstport`. |
| 16.18 | Rótulos DNS longos para revisão | PCAP | Filtro `dns.qry.name matches "^[a-zA-Z0-9]{20,}\\."`. Campos `frame.number`, `dns.qry.name`. Linhas únicas. |

## Categoria 17

**Forense Avançada**

| ID | Ação | Entrada | Operação padrão |
| --- | --- | --- | --- |
| 17.1 | Linha do tempo de pacotes | PCAP | Campos `frame.number`, `frame.time_epoch`, `ip.src`, `ipv6.src`, `ip.dst`, `ipv6.dst`, `frame.protocols`. |
| 17.2 | Seguir uma sessão TCP (ASCII) | PCAP | Segue stream TCP em ASCII. Configure o número em **a → s**. |
| 17.5 | Hashes do arquivo de captura para cadeia de custódia | PCAP | Metadados com `capinfos` (`-H`). |
| 17.6 | Evidências de sequência, ACK e payload TCP | PCAP | Filtro `tcp`. Campos `frame.number`, `tcp.stream`, `tcp.seq`, `tcp.ack`, `tcp.len`, `tcp.payload`. |
| 17.7 | Quadros com deltas de timestamp negativos | PCAP | Filtro `frame.time_delta < 0`. Campos `frame.number`, `frame.time_epoch`, `frame.time_delta`. |
| 17.9 | RTT observado de confirmações TCP | PCAP | Filtro `tcp.analysis.ack_rtt`. Campos `frame.number`, `tcp.stream`, `tcp.analysis.ack_rtt`. |
| 17.10 | Opções de timestamp TCP observadas | PCAP | Filtro `tcp.options.timestamp.tsval`. Campos `frame.time_epoch`, `tcp.stream`, `tcp.options.timestamp.tsval`, `tcp.options.timestamp.tsecr`. |
| 17.11 | Exportar mensagens de e-mail capturadas | PCAP | Escreve arquivos em uma pasta de exportação com o dissector `imf`. |
| 17.12 | Eventos observados de navegação HTTP | PCAP / log HTTP | Filtro `http.request`. Campos `frame.time_epoch`, `http.host`, `http.request.uri`, `http.response_in`. Em log HTTP, usa a consulta nativa. |

## Categoria 18

**Documentos (PDF/Office/Imagens)**

| ID | Ação | Entrada | Operação padrão |
| --- | --- | --- | --- |
| 18.2 | Extrair texto do PDF | PDF | `pdftotext`. |
| 18.5 | Consultar metadados do PDF | PDF | `pdfinfo`. |
| 18.6 | Listar imagens do PDF | PDF | `pdfimages -list`. |

## Categoria 19

**Conversões & Extração**

| ID | Ação | Entrada | Operação padrão |
| --- | --- | --- | --- |
| 19.4 | Pacotes em hexadecimal | PCAP | Saída hexadecimal. |
| 19.5 | Dissecação de pacotes em JSON | PCAP | Saída JSON. |
| 19.6 | Metadados de quadros em CSV | PCAP | Saída CSV. |
| 19.7 | Dissecação de pacotes em XML (PDML) | PCAP | Saída PDML. |
| 19.8 | Salvar pacotes que correspondem ao filtro de exibição | PCAP | Grava `filtered.pcapng` usando o filtro de exibição informado. |
| 19.14 | Gravar uma cópia em PCAPNG | PCAP | Grava `filtered.pcapng` usando o filtro de exibição informado. |
| 19.19 | Gerar PCAPNG derivado de um log (automático) | Log de texto | Gera um PCAPNG derivado com detecção do formato de entrada. |
| 19.20 | Gerar PCAPNG derivado preservando os bytes do log | Log de texto | Gera um PCAPNG derivado com bytes genéricos. |
| 19.21 | Ler dados de texto de uma captura derivada | PCAP | Filtro `data.data`. Campos `data.data`. |
| 19.22 | Inspecionar chaves e caminhos JSON de uma captura derivada | PCAP | Filtro `json`. Campos `frame.number`, `json.key`, `json.path`. |

## Categoria 20

**Interação com Serviços**

Categoria sem preset executável nesta versão.

## Categoria 21

**Relatório & Exportação**

| ID | Ação | Entrada | Operação padrão |
| --- | --- | --- | --- |
| 21.2 | Gerar dissecação de pacotes como JSON | PCAP | Saída JSON. |
| 21.4 | Timeline de tráfego | PCAP / log HTTP | Campos `frame.time_epoch`, `frame.number`, `frame.protocols`, `frame.len`. Em log HTTP, usa a consulta nativa. |
| 21.5 | Nomes DNS observados para triagem | PCAP | Filtro `dns.flags.response == 0 && dns.qry.name`. Campos `dns.qry.name`. Linhas únicas. |
| 21.6 | Estatísticas de resumo da captura | PCAP / log HTTP | Estatísticas TShark `io,phs`, `conv,tcp`. Em log HTTP, usa a consulta nativa. |
| 21.8 | Hashes do arquivo de captura | PCAP | Metadados com `capinfos` (`-H`). |

## Categoria 22

**Aprender & Tutor**

Guia integrado também acessível com **F1**.

## Categoria 23

**Workspace & Projetos**

Projetos, notas, arquivos vinculados, histórico, favoritos e idioma. Veja [Projetos e arquivos](manual.md#projetos-e-arquivos).

## Categoria 24

**BGP / Roteamento**

| ID | Ação | Entrada | Operação padrão |
| --- | --- | --- | --- |
| 24.1 | Tipos de mensagens BGP | PCAP | Filtro `bgp`. Campos `frame.number`, `ip.src`, `ipv6.src`, `bgp.type`. |

## Categoria 25

**SNMP**

| ID | Ação | Entrada | Operação padrão |
| --- | --- | --- | --- |
| 25.1 | Inventário de pacotes SNMP | PCAP | Filtro `snmp`. Campos `frame.number`, `ip.src`, `ipv6.src`, `snmp.version`. |

## Categoria 26

**Industrial / SCADA / ICS**

| ID | Ação | Entrada | Operação padrão |
| --- | --- | --- | --- |
| 26.1 | Códigos de função Modbus/TCP | PCAP | Filtro `modbus`. Campos `frame.number`, `ip.src`, `ip.dst`, `modbus.func_code`. |

## Categoria 27

**Bluetooth / BLE**

| ID | Ação | Entrada | Operação padrão |
| --- | --- | --- | --- |
| 27.1 | Quadros Bluetooth Low Energy | PCAP | Filtro `btle`. Campos `frame.number`, `frame.protocols`, `btle.access_address`. |

## Categoria 34

**Análise de Identidade & SSO**

| ID | Ação | Entrada | Operação padrão |
| --- | --- | --- | --- |
| 34.1 | Tipos de mensagem Kerberos | PCAP | Filtro `kerberos.msg_type`. Campos `frame.number`, `ip.src`, `ipv6.src`, `kerberos.msg_type`. |
| 34.2 | Indícios observáveis de endpoints OAuth em HTTP | PCAP | Filtro `http.request.uri contains "oauth"`. Campos `frame.number`, `ip.src`, `ipv6.src`, `http.host`. |
| 34.3 | Presença de marcador SAML em HTTP visível | PCAP | Filtro `http contains "SAMLResponse" \|\| http contains "SAMLRequest"`. Campos `frame.number`, `ip.src`, `ipv6.src`, `http.host`. |
| 34.5 | Requisições Kerberos TGS para revisão | PCAP | Filtro `kerberos.msg_type == 12`. Campos `frame.number`, `ip.src`, `ipv6.src`, `kerberos.realm`, `kerberos.sname_string`. |

Os padrões vêm de [catalog.json](../src/actions/catalog.json). Os IDs pendentes vêm de [requested.json](../src/actions/requested.json).

[Voltar ao manual](manual.md)
