package teste.lucasvegi.pokemongooffline.Model;

/**
 * Classe que representa um tipo de doce usado para evoluir Pokémon
 */
public class Doce {
    private int id;
    private String nome;

    public Doce(int id, String nome) {
        this.id = id;
        this.nome = nome;
    }

    public int getId() {
        return id;
    }

    public void setId(int id) {
        this.id = id;
    }

    public String getNome() {
        return nome;
    }

    public void setNome(String nome) {
        this.nome = nome;
    }

    @Override
    public String toString() {
        return nome;
    }
}
