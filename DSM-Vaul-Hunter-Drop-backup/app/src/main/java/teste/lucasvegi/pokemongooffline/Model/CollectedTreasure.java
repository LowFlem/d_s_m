package teste.lucasvegi.pokemongooffline.Model;

import java.util.Date;

/**
 * Class representing a Treasure collected by a user
 */
public class CollectedTreasure {
    private int id;
    private Treasure treasure;
    private Usuario user;
    private int cp;
    private int hp;
    private double weight;
    private double height;
    private Date collectionDate;
    private double latitude;
    private double longitude;

    // Complete constructor
    public CollectedTreasure(int id, Treasure treasure, Usuario user, int cp, int hp, 
                           double weight, double height, Date collectionDate, 
                           double latitude, double longitude) {
        this.id = id;
        this.treasure = treasure;
        this.user = user;
        this.cp = cp;
        this.hp = hp;
        this.weight = weight;
        this.height = height;
        this.collectionDate = collectionDate;
        this.latitude = latitude;
        this.longitude = longitude;
    }

    // Constructor without ID (for new collections)
    public CollectedTreasure(Treasure treasure, Usuario user, int cp, int hp, 
                           double weight, double height, Date collectionDate, 
                           double latitude, double longitude) {
        this.treasure = treasure;
        this.user = user;
        this.cp = cp;
        this.hp = hp;
        this.weight = weight;
        this.height = height;
        this.collectionDate = collectionDate;
        this.latitude = latitude;
        this.longitude = longitude;
    }

    public int getId() {
        return id;
    }

    public void setId(int id) {
        this.id = id;
    }

    public Treasure getTreasure() {
        return treasure;
    }

    public void setTreasure(Treasure treasure) {
        this.treasure = treasure;
    }

    public Usuario getUser() {
        return user;
    }

    public void setUser(Usuario user) {
        this.user = user;
    }

    public int getCp() {
        return cp;
    }

    public void setCp(int cp) {
        this.cp = cp;
    }

    public int getHp() {
        return hp;
    }

    public void setHp(int hp) {
        this.hp = hp;
    }

    public double getWeight() {
        return weight;
    }

    public void setWeight(double weight) {
        this.weight = weight;
    }

    public double getHeight() {
        return height;
    }

    public void setHeight(double height) {
        this.height = height;
    }

    public Date getCollectionDate() {
        return collectionDate;
    }

    public void setCollectionDate(Date collectionDate) {
        this.collectionDate = collectionDate;
    }

    public double getLatitude() {
        return latitude;
    }

    public void setLatitude(double latitude) {
        this.latitude = latitude;
    }

    public double getLongitude() {
        return longitude;
    }

    public void setLongitude(double longitude) {
        this.longitude = longitude;
    }

    /**
     * Returns the formatted height of the Treasure
     * @return Formatted height (m)
     */
    public String getFormattedHeight() {
        return String.format("%.2f m", height);
    }

    /**
     * Returns the formatted weight of the Treasure
     * @return Formatted weight (kg)
     */
    public String getFormattedWeight() {
        return String.format("%.2f kg", weight);
    }

    /**
     * Checks if the Treasure is large (XL)
     * @return true if the Treasure is large
     */
    public boolean isXL() {
        return weight > treasure.getWeight() * 1.15 || height > treasure.getHeight() * 1.15;
    }

    /**
     * Checks if the Treasure is small (XS)
     * @return true if the Treasure is small
     */
    public boolean isXS() {
        return weight < treasure.getWeight() * 0.85 || height < treasure.getHeight() * 0.85;
    }
}