package teste.lucasvegi.pokemongooffline.Model;

/**
 * Classe que representa um usuário do aplicativo
 */
public class Usuario {
    private String login;
    private String nome;
    private String senha;
    private char sexo; // 'M' para masculino, 'F' para feminino
    private int pontos;
    private int nivel;

    public Usuario(String login, String nome, String senha, char sexo) {
        this.login = login;
        this.nome = nome;
        this.senha = senha;
        this.sexo = sexo;
        this.pontos = 0;
        this.nivel = 1;
    }

    public Usuario(String login, String nome, String senha, char sexo, int pontos, int nivel) {
        this.login = login;
        this.nome = nome;
        this.senha = senha;
        this.sexo = sexo;
        this.pontos = pontos;
        this.nivel = nivel;
    }

    public String getLogin() {
        return login;
    }

    public void setLogin(String login) {
        this.login = login;
    }

    public String getNome() {
        return nome;
    }

    public void setNome(String nome) {
        this.nome = nome;
    }

    public String getSenha() {
        return senha;
    }

    public void setSenha(String senha) {
        this.senha = senha;
    }

    public char getSexo() {
        return sexo;
    }

    public void setSexo(char sexo) {
        this.sexo = sexo;
    }

    public int getPontos() {
        return pontos;
    }

    public void setPontos(int pontos) {
        this.pontos = pontos;
    }

    public int getNivel() {
        return nivel;
    }

    public void setNivel(int nivel) {
        this.nivel = nivel;
    }

    /**
     * Adiciona pontos ao usuário e atualiza o nível se necessário
     * @param pontosAdicionais Pontos a serem adicionados
     */
    public void adicionarPontos(int pontosAdicionais) {
        this.pontos += pontosAdicionais;
        atualizarNivel();
    }

    /**
     * Atualiza o nível do usuário com base nos pontos
     * A cada 1000 pontos, o usuário sobe um nível
     */
    private void atualizarNivel() {
        int novoNivel = (this.pontos / 1000) + 1;
        if (novoNivel > this.nivel) {
            this.nivel = novoNivel;
        }
    }
}
