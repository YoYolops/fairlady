Uma rede centralizada mas resilinte à perda do no central. Replica dados da maquina host em um determinado ponto.
Se conecta com nos via tcp e todos atualizam pra versao mais recente dos dados do no mestre
Nos filhos podem eviar dados que serao sincronizados pela rede atraves do mestre
Caso um no filho esteja fora do ar, quando voltar, aplica as alteracoes
O no mestre sabe a porcentagem de rede atualizada
Caso no mestre caia, um nó filho assume como mestre (por voto talvez?)