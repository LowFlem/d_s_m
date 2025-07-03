package teste.lucasvegi.pokemongooffline.Model;

import java.util.Date;

/**
 * Classe que representa um Pokémon capturado por um usuário
 */
public class PokemonCapturado {
    private int id;
    private Pokemon pokemon;
    private Usuario usuario;
    private int cp;
    private int hp;
    private double peso;
    private double altura;
    private Date dataCaptura;
    private double latitude;
    private double longitude;

    // Construtor completo
    public PokemonCapturado(int id, Pokemon pokemon, Usuario usuario, int cp, int hp, 
                           double peso, double altura, Date dataCaptura, 
                           double latitude, double longitude) {
        this.id = id;
        this.pokemon = pokemon;
        this.usuario = usuario;
        this.cp = cp;
        this.hp = hp;
        this.peso = peso;
        this.altura = altura;
        this.dataCaptura = dataCaptura;
        this.latitude = latitude;
        this.longitude = longitude;
    }

    // Construtor sem ID (para novas capturas)
    public PokemonCapturado(Pokemon pokemon, Usuario usuario, int cp, int hp, 
                           double peso, double altura, Date dataCaptura, 
                           double latitude, double longitude) {
        this.pokemon = pokemon;
        this.usuario = usuario;
        this.cp = cp;
        this.hp = hp;
        this.peso = peso;
        this.altura = altura;
        this.dataCaptura = dataCaptura;
        this.latitude = latitude;
        this.longitude = longitude;
    }

    public int getId() {
        return id;
    }

    public void setId(int id) {
        this.id = id;
    }

    public Pokemon getPokemon() {
        return pokemon;
    }

    public void setPokemon(Pokemon pokemon) {
        this.pokemon = pokemon;
    }

    public Usuario getUsuario() {
        return usuario;
    }

    public void setUsuario(Usuario usuario) {
        this.usuario = usuario;
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

    public Date getDataCaptura() {
        return dataCaptura;
    }

    public void setDataCaptura(Date dataCaptura) {
        this.dataCaptura = dataCaptura;
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
     * Retorna a altura do Pokémon formatada
     * @return Altura formatada (m)
     */
    public String getAlturaFormatada() {
        return String.format("%.2f m", altura);
    }

    /**
     * Retorna o peso do Pokémon formatado
     * @return Peso formatado (kg)
     */
    public String getPesoFormatado() {
        return String.format("%.2f kg", peso);
    }

    /**
     * Verifica se o Pokémon é grande (XL)
     * @return true se o Pokémon é grande
     */
    public boolean isXL() {
        return peso > pokemon.getPeso() * 1.15 || altura > pokemon.getAltura() * 1.15;
    }

    /**
     * Verifica se o Pokémon é pequeno (XS)
     * @return true se o Pokémon é pequeno
     */
    public boolean isXS() {
        return peso < pokemon.getPeso() * 0.85 || altura < pokemon.getAltura() * 0.85;
    }
}
