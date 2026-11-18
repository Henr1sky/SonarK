# SonarK

**Análise offline de tráfego e logs no terminal, com ações guiadas para TShark.**

SonarK é uma CLI/TUI para investigar arquivos que você já tem: capturas PCAP/PCAPNG, logs locais e PDFs anexados. Você escolhe a entrada, abre uma ação pelo nome ou ID, ajusta os filtros e consulta o resultado na própria interface. O projeto guarda histórico, favoritos, presets de saída e relatórios.

**Versão documentada:** 0.0.10 · **Plataforma:** Linux · **Idiomas da interface:** inglês e português · **Licença:** [MIT](LICENSE)

## O que você pode fazer

| Entrada | Uso típico | Primeiras ações |
| --- | --- | --- |
| PCAP ou PCAPNG | Inventariar protocolos, observar fluxos e explorar pacotes com TShark | **[1.1]** Hierarquia de protocolos, **[10.1]** Consultas DNS, **[3.7]** Frequência de portas de destino em SYN TCP |
| Log HTTP Common ou Combined | Contar códigos de status, consultar URIs e montar uma linha do tempo | **[1.4]** Resumo do log, **[5.7]** Códigos de status, **[21.4]** Linha do tempo |
| Log genérico, CSV ou JSON/JSONL | Ler e buscar texto ou gerar PCAPNG derivado para usar os dissectors disponíveis | **[19.19]** Conversão automática, **[19.20]** Preservação de bytes, **[19.21]** Leitura de dados de texto, **[19.22]** Chaves JSON |
| PDF anexado | Consultar texto, metadados e imagens presentes no documento | **[18.2]** Texto, **[18.5]** Metadados, **[18.6]** Lista de imagens |

O catálogo tem **233 ações executáveis**. Dessas, **20** também consultam logs HTTP diretamente. O editor completo de comandos e etapas está disponível em **[2.2]**, **[3.7]** e **[4.6]**. Dentro de uma categoria, **i** mostra os IDs que ainda estão sem preset executável.

## Instalação e primeiros passos

Instale Git e uma toolchain Rust com suporte à edição 2024. A versão foi validada com Rust **1.97.1**. Para analisar capturas, coloque `tshark` e `capinfos` no `PATH`. A conversão de logs usa `text2pcap`; as ações de PDF usam ferramentas do Poppler. Consulte a [lista de dependências e como verificá-las](docs/manual.md#instalação-e-requisitos).

Com acesso ao repositório, rode:

```bash
git clone https://github.com/Henr1sky/SonarK.git
cd SonarK
cargo install --locked --path .
sonark --version
sonark
```

Se `sonark` não estiver no `PATH`, use `~/.cargo/bin/sonark` ou veja o [passo a passo da instalação](docs/manual.md#instalar-o-binário-para-uso-diário). Para executar a partir do código, use `cargo run --locked`. Para abrir um arquivo com o binário instalado:

```bash
sonark --project investigacao "/caminho/captura.pcapng"
```

1. Pressione qualquer tecla na abertura. Use **p** para selecionar ou trocar o arquivo do projeto.
2. Use **↑/↓** para navegar e **Enter** ou **→** para abrir uma categoria e uma ação.
3. Na ação, pressione **h** para entender o preset, **a** para ver os ajustes e **Enter** para executar.
4. Use **x** para exportar um resultado. **F1** abre o guia geral, e **?** mostra os atalhos.
5. Para mudar o idioma ou gerenciar projetos, abra **[23] Workspace & Projects**.

## O que o SonarK não faz

- Não captura tráfego ao vivo, não reproduz tráfego na rede e não executa varreduras ou exploração. A entrada é um arquivo local.
- Não recupera pacotes originais a partir de um log. A conversão escreve uma **representação sintética** dos registros. Filtros que dependem de Ethernet, IP ou TCP reais precisam de uma captura com esses dados.
- Não determina sozinho que um host é atacante ou que houve comprometimento. Os resultados descrevem os registros e pacotes observados. Uma contagem de SYN pode incluir retransmissões.
- Não oferece a dissecação gráfica completa do Wireshark nem edição livre de scripts de shell. O editor combina argumentos do preset e etapas de saída definidas.
- Os projetos referenciam os arquivos originais pelo caminho. Histórico e exportações podem conter conteúdo sensível e são armazenados em arquivos locais legíveis.

## Documentação

- [Manual de uso](docs/manual.md) — instalação, projetos, navegação, exemplos reproduzíveis, edição, relatórios, armazenamento e diagnóstico.
- [Catálogo de ações](docs/catalogo.md) — lista por categoria com IDs, entradas, filtros, campos e operações.
- [Licença MIT](LICENSE) — termos de uso e distribuição.
