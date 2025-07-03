package teste.lucasvegi.pokemongooffline.Model;

/**
 * Class representing a type of crystal used to upgrade Treasures
 */
public class Crystal {
    private int id;
    private String name;

    public Crystal(int id, String name) {
        this.id = id;
        this.name = name;
    }

    public int getId() {
        return id;
    }

    public void setId(int id) {
        this.id = id;
    }

    public String getName() {
        return name;
    }

    public void setName(String name) {
        this.name = name;
    }

    @Override
    public String toString() {
        return name;
    }
}