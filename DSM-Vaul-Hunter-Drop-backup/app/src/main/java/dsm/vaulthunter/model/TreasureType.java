package dsm.vaulthunter.model;

/**
 * Class representing a Treasure type or category in the Vault Hunter game
 */
public class TreasureType {
    private int id;
    private String name;
    private String description;
    private int iconResource;

    public TreasureType(int id, String name, String description, int iconResource) {
        this.id = id;
        this.name = name;
        this.description = description;
        this.iconResource = iconResource;
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

    public String getDescription() {
        return description;
    }

    public void setDescription(String description) {
        this.description = description;
    }

    public int getIconResource() {
        return iconResource;
    }

    public void setIconResource(int iconResource) {
        this.iconResource = iconResource;
    }

    @Override
    public String toString() {
        return name;
    }
}