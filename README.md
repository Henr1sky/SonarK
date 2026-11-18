# SonarK

SonarK é uma interface de terminal para análise offline de capturas PCAP/PCAPNG, logs de texto e PDFs anexados. Os presets mostram a saída na própria interface.

## Executar no Linux

O projeto usa Rust 1.97.1, Ratatui e Crossterm. Tenha `tshark` e `capinfos` no `PATH` para analisar capturas e `text2pcap` para converter logs. As ações de PDF usam `pdftotext`, `pdfinfo` e `pdfimages` do Poppler.

```bash
cargo run --offline
cargo run --offline -- --project investigacao "/caminho/minha captura.pcapng"
```

Remova `--offline` na primeira compilação se o cache do Cargo estiver vazio. Sem argumentos, o SonarK abre o projeto `default`. Use **p** para escolher a entrada do projeto, que pode ser um log de texto ou uma captura de pacotes.

## Operar

| Onde | Teclas |
| --- | --- |
| Menu | **↑/↓** ou **j/k** para navegar, **Enter** para abrir |
| Lista de ações | **/** busca por nome ou ID, **Enter** abre a ação e **i** mostra os IDs enviados que ainda estão sem preset executável |
| Ação | **Enter** executa, **h** abre a ajuda contextual, **e** mostra os detalhes do preset e **a** abre ajustes |
| Ajustes | **f** edita o filtro, **n** limita linhas e **m** alterna ordem original, linhas únicas ou contagem. **g** agrupa por um campo permitido |
| Ajustes TCP | **p** escolhe a porta, **r** muda a direção, **u** alterna o filtro PSH e **d** decodifica o payload quando disponível. No follow, **s** escolhe o stream |
| Captura derivada | Após converter um log, **o** ativa o PCAPNG para os presets. No Workspace, **v** alterna entre ele e o log original |
| Ação | **s** salva um preset para essa ação no projeto, **b** carrega, **f** alterna favorito |
| Saída | **↑/↓**, **←/→**, **PgUp/PgDn** rolam a saída. **x** exporta após uma execução bem-sucedida |
| Execução | **Esc** ou **Ctrl+C** cancela a análise |
| Workspace [23] | **n** lista os projetos, **r** renomeia o ativo, **m** edita a nota, **y** abre histórico, **f** abre favoritos e **Enter** alterna o idioma |
| Lista de projetos | **↑/↓** seleciona, **Enter** abre, **c** inicia a criação |
| Novo projeto | Digite o nome. Use **p** para escolher um log de texto ou PCAP e **a** para arquivos de apoio em texto. **Enter** confirma a criação |
| Arquivos de apoio | No Workspace, use **u** para listar e **a** para vincular arquivos. **Enter** abre arquivos de texto. A categoria **[18]** usa o primeiro PDF anexado |
| Navegador de arquivos | **p** abre o navegador. **→** ou **Enter** abre a pasta selecionada, **←** ou **Backspace** sobe uma pasta, **d** abre Downloads, **~** abre sua pasta pessoal, **w** abre a pasta de trabalho, **t** edita o caminho e **a** alterna a visibilidade de arquivos ocultos |
| Log HTTP | **g** lê o log vinculado, **/** busca texto e **Esc** volta |
| Guia | **F1** abre o guia dentro do SonarK. **?** lista os atalhos |

Em PCAP, o filtro de exibição adicional é combinado com o filtro do preset. Para filtrar por tempo, use `frame.time_relative >= 10 && frame.time_relative <= 60`. Em log, o filtro procura texto na linha original, sem distinguir maiúsculas de minúsculas. **[22] Learn & Tutor** traz exemplos.

A exportação comum gera texto, JSON com os metadados da análise, CSV com o número e o texto de cada linha, ou Markdown. Nos presets de conversão **[19.4] a [19.7]**, **x → 5** salva a saída no formato produzido pelo TShark quando o resultado foi mantido na íntegra. Os presets de extração de objetos e **[19.8] Salvar pacotes que correspondem ao filtro de exibição** gravam em uma nova pasta em `exports/<categoria>/`.

O SonarK salva os caminhos dos arquivos no projeto e abre os originais diretamente. Se mover um PCAP ou log, use o navegador para selecionar o novo caminho.

## Estudar um access log

Salve o arquivo localmente, vincule-o com **p** e use **g** para voltar ao log vinculado. O leitor mostra as linhas numeradas e conta os status HTTP encontrados no formato comum de logs de acesso. A busca ignora maiúsculas e minúsculas. Para arquivos longos, a interface sinaliza quando está mostrando apenas os primeiros 16 MiB.

Com um log como entrada do projeto, as ações **[LOG]** rodam diretamente nas categorias. Experimente **[1.4] Resumo do log de acesso**, **[1.12] Requisições por endereço de cliente**, **[5.7] Códigos de status HTTP** e **[21.4] Linha do tempo HTTP**. O resultado pode ser revisitado no histórico ou exportado com **x**. As ações **[PCAP]** exigem uma captura de pacotes.

## Converter um log para uso com o TShark

1. Vincule o arquivo local com **p**.
2. Em **[19] Conversões & Extração**, abra **[19.19] Gerar PCAPNG derivado de um log (automático)** e pressione **Enter**. A ação **[19.20]** preserva os bytes da entrada como dados genéricos.
3. No resultado, pressione **o** para analisar a captura gerada com os presets do TShark. O log continua disponível com **g**. No Workspace, **v** alterna a entrada ativa.

| Entrada | Representação | Presets para experimentar |
| --- | --- | --- |
| Log HTTP Common ou Combined | Mensagens HTTP sintéticas com horário e campos registrados | **[5.6]** Métodos, **[5.7]** Códigos de status |
| JSON ou JSONL com objetos ou arrays compatíveis | Registros entregues ao dissecador JSON | **[19.22]** Chaves e caminhos JSON |
| Outros logs em texto, inclusive CSV | Bytes do arquivo como dados genéricos | **[19.21]** com **a → d** para ler em texto |

O resultado é um PCAPNG derivado e marcado como sintético. Ele não contém pacotes originais, tráfego TCP/IP reconstruído nem corpos HTTP que estavam ausentes no log. Para texto genérico e JSON, os horários da captura são gerados na conversão. Os arquivos `log-derived.pcapng` e `source.json` ficam em uma pasta própria sob `exports/19/`. O limite atual de conversão é **64 MiB** por arquivo local de texto.

## Ajustar a saída de uma captura de pacotes

- **[2.2] Respondentes TCP observados (SYN+ACK)** conta os pacotes por endereço de origem, com os maiores totais primeiro. Esse total descreve os SYN+ACK vistos na captura.
- **[4.6] Payload TCP por porta**. Abra **a**, use **p** para informar a porta, **r** para escolher **Para a porta**, **u** para filtrar PSH e **d** para decodificar. O resultado mostra o IP de origem, a porta de origem, a porta de destino e o texto do segmento. Para incluir os dois sentidos, deixe **r** em **Ambas**.

## Editor de comando e etapas de saída

As ações **[3.7]**, **[2.2]** e **[4.6]** têm o novo editor. A tela mostra o preset original e o comando que será executado.

- **a → 1** e **a → 2** definem os endereços de origem e destino. **a → e** edita o filtro base `-Y`, **a → E** edita os campos `-e` e **a → f** acrescenta outro filtro. O comando acompanha a edição.
- **t** abre as etapas de saída. **a** adiciona uma etapa pronta, **e** edita, **Espaço** ativa ou desativa, **d** remove e **< / >** muda a ordem. **r** recupera a sequência gerada pelos ajustes guiados.
- O menu oferece **sort**, **uniq**, **head**, **tail**, **grep**, **cut**, **wc** e receitas de **awk**. A ordenação por data usa valores ISO com formato e fuso uniformes. Em modo guiado, **a → n** aceita um limite livre e **a → 5/6/7** define 10, 20 ou 50 linhas.
- **c → c** envia o comando à área de transferência de terminais com OSC 52. A janela **c** exibe também o comando completo para seleção manual. **s** salva, **b** carrega e **R** restaura o original. Ao sair com alterações, a tela oferece salvar ou descartar.
- A saída bruta aparece enquanto o TShark roda. As etapas que dependem da saída inteira, como ordenação ou contagem, aparecem ao final. **B** deixa a análise em segundo plano. Reabra a ação para acompanhá-la ou veja o resultado no histórico.
- Alterar as etapas depois de executar reaproveita as linhas coletadas. Alterar o filtro ou os endereços exige executar novamente para ler os pacotes correspondentes.

## Configuração e projetos

- O idioma fica em `$XDG_CONFIG_HOME/sonark/config.json`, com padrão em `~/.config/sonark/config.json`.
- Projetos ficam em `$XDG_DATA_HOME/sonark/projects/saves/<projeto>/`, com padrão sob `~/.local/share/`. O histórico, os presets, os favoritos e a nota ficam em `project.json`. Os relatórios ficam em `exports/<categoria>/`.
- Cada execução retém até 2 MiB de saída e o projeto guarda as 20 últimas análises concluídas. Contagem, limitação e deduplicação na interface operam sobre a saída retida. A interface sinaliza o corte.
- A nota e o histórico são gravados em texto legível no JSON. Saídas e relatórios podem conter dados sensíveis presentes na captura.

A v0.0.10 oferece 233 presets executáveis, dos quais 20 também rodam em logs HTTP. Pressione **i** nas categorias para consultar os IDs recebidos que ainda estão sem preset executável.
