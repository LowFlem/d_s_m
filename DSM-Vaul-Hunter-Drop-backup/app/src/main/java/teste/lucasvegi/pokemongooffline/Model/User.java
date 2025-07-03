package teste.lucasvegi.pokemongooffline.Model;

/**
 * Class representing a user of the application
 */
public class User {
    private String login;
    private String name;
    private String password;
    private char gender; // 'M' for male, 'F' for female
    private int points;
    private int level;

    public User(String login, String name, String password, char gender) {
        this.login = login;
        this.name = name;
        this.password = password;
        this.gender = gender;
        this.points = 0;
        this.level = 1;
    }

    public User(String login, String name, String password, char gender, int points, int level) {
        this.login = login;
        this.name = name;
        this.password = password;
        this.gender = gender;
        this.points = points;
        this.level = level;
    }

    public String getLogin() {
        return login;
    }

    public void setLogin(String login) {
        this.login = login;
    }

    public String getName() {
        return name;
    }

    public void setName(String name) {
        this.name = name;
    }

    public String getPassword() {
        return password;
    }

    public void setPassword(String password) {
        this.password = password;
    }

    public char getGender() {
        return gender;
    }

    public void setGender(char gender) {
        this.gender = gender;
    }

    public int getPoints() {
        return points;
    }

    public void setPoints(int points) {
        this.points = points;
    }

    public int getLevel() {
        return level;
    }

    public void setLevel(int level) {
        this.level = level;
    }

    /**
     * Adds points to the user and updates the level if necessary
     * @param additionalPoints Points to be added
     */
    public void addPoints(int additionalPoints) {
        this.points += additionalPoints;
        updateLevel();
    }

    /**
     * Updates the user's level based on points
     * Every 1000 points, the user advances one level
     */
    private void updateLevel() {
        int newLevel = (this.points / 1000) + 1;
        if (newLevel > this.level) {
            this.level = newLevel;
        }
    }
}