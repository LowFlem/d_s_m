package teste.lucasvegi.pokemongooffline.Model;

/**
 * Classe que representa um Pokémon no jogo
 */
public class Pokemon {
    private int numero;
    private String nome;
    private String descricao;
    private int cpBase;
    private int hpBase;
    private double peso;
    private double altura;
    private int raridade; // 1 (comum) a 5 (lendário)
    private Tipo tipoPrimario;
    private Tipo tipoSecundario;
    private Doce doce;
    private double distancia; // Distância para chocar ovo (em km)
    private Integer candyEvolucao; // Quantidade de doces para evoluir
    private Pokemon evolucao; // Próxima evolução

    public Pokemon(int numero, String nome, String descricao, int cpBase, int hpBase, 
                  double peso, double altura, int raridade, Tipo tipoPrimario, 
                  Tipo tipoSecundario, Doce doce, double distancia) {
        this.numero = numero;
        this.nome = nome;
        this.descricao = descricao;
        this.cpBase = cpBase;
        this.hpBase = hpBase;
        this.peso = peso;
        this.altura = altura;
        this.raridade = raridade;
        this.tipoPrimario = tipoPrimario;
        this.tipoSecundario = tipoSecundario;
        this.doce = doce;
        this.distancia = distancia;
    }

    public int getNumero() {
        return numero;
    }

    public void setNumero(int numero) {
        this.numero = numero;
    }

    public String getNome() {
        return nome;
    }

    public void setNome(String nome) {
        this.nome = nome;
    }

    public String getDescricao() {
        return descricao;
    }

    public void setDescricao(String descricao) {
        this.descricao = descricao;
    }

    public int getCpBase() {
        return cpBase;
    }

    public void setCpBase(int cpBase) {
        this.cpBase = cpBase;
    }

    public int getHpBase() {
        return hpBase;
    }

    public void setHpBase(int hpBase) {
        this.hpBase = hpBase;
    }

    public double getPeso() {
        return peso;
    }

    public void setPeso(double peso) {
        this.peso = peso;
    }

    public double getAltura() {
        return altura;
    }

    public void setAltura(double altura) {
        this.altura = altura;
    }

    public int getRaridade() {
        return raridade;
    }

    public void setRaridade(int raridade) {
        this.raridade = raridade;
    }

    public Tipo getTipoPrimario() {
        return tipoPrimario;
    }

    public void setTipoPrimario(Tipo tipoPrimario) {
        this.tipoPrimario = tipoPrimario;
    }

    public Tipo getTipoSecundario() {
        return tipoSecundario;
    }

    public void setTipoSecundario(Tipo tipoSecundario) {
        this.tipoSecundario = tipoSecundario;
    }

    public Doce getDoce() {
        return doce;
    }

    public void setDoce(Doce doce) {
        this.doce = doce;
    }

    public double getDistancia() {
        return distancia;
    }

    public void setDistancia(double distancia) {
        this.distancia = distancia;
    }

    public Integer getCandyEvolucao() {
        return candyEvolucao;
    }

    public void setCandyEvolucao(Integer candyEvolucao) {
        this.candyEvolucao = candyEvolucao;
    }

    public Pokemon getEvolucao() {
        return evolucao;
    }

    public void setEvolucao(Pokemon evolucao) {
        this.evolucao = evolucao;
    }
    
    /**
     * Verifica se o Pokémon pode evoluir
     * @return true se o Pokémon pode evoluir
     */
    public boolean podeEvoluir() {
        return evolucao != null && candyEvolucao != null;
    }
    
    /**
     * Retorna o nome do estágio evolutivo do Pokémon
     * @return Nome do estágio evolutivo
     */
    public String getEstagioEvolutivo() {
        // Verifica se é o primeiro estágio
        if (numero % 3 == 1) {
            return "Primeiro estágio";
        } 
        // Verifica se é o último estágio
        else if (!podeEvoluir()) {
            return "Estágio final";
        } 
        // Caso contrário, é um estágio intermediário
        else {
            return "Estágio intermediário";
        }
    }
    
    /**
     * Retorna a string com os tipos do Pokémon
     * @return String com o(s) tipo(s)
     */
    public String getTiposString() {
        if (tipoSecundario != null) {
            return tipoPrimario.getNome() + " / " + tipoSecundario.getNome();
        } else {
            return tipoPrimario.getNome();
        }
    }

    @Override
    public String toString() {
        return "#" + numero + " " + nome;
    }

    public int getQuantDocesNecessarios() {
        return 0;
    }
}
