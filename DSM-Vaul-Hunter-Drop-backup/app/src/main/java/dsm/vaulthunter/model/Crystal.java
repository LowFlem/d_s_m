package dsm.vaulthunter.model;

/**
 * Class representing a Crystal used to upgrade treasures in the Vault Hunter game
 */
public class Crystal {
    private int id;
    private String name;
    private TreasureType type;
    private int quantity;
    private int iconResource;
    
    public Crystal(int id, String name, TreasureType type, int iconResource) {
        this.id = id;
        this.name = name;
        this.type = type;
        this.quantity = 0;
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
    
    public TreasureType getType() {
        return type;
    }
    
    public void setType(TreasureType type) {
        this.type = type;
    }
    
    public int getQuantity() {
        return quantity;
    }
    
    public void setQuantity(int quantity) {
        this.quantity = quantity;
    }
    
    public void addQuantity(int amount) {
        this.quantity += amount;
    }
    
    public boolean useQuantity(int amount) {
        if (quantity >= amount) {
            quantity -= amount;
            return true;
        }
        return false;
    }
    
    public int getIconResource() {
        return iconResource;
    }
    
    public void setIconResource(int iconResource) {
        this.iconResource = iconResource;
    }
    
    @Override
    public String toString() {
        return name + " (x" + quantity + ")";
    }
}