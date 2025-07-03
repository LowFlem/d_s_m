package teste.lucasvegi.pokemongooffline.Model;

import java.util.Date;

/**
 * Classe que representa o aparecimento de um Pokémon no mapa
 */
public class Aparecimento {
    private int id;
    private Pokemon pokemon;
    private double latitude;
    private double longitude;
    private Date dataHora;
    private boolean ativo;

    // Construtor completo
    public Aparecimento(int id, Pokemon pokemon, double latitude, double longitude, Date dataHora, boolean ativo) {
        this.id = id;
        this.pokemon = pokemon;
        this.latitude = latitude;
        this.longitude = longitude;
        this.dataHora = dataHora;
        this.ativo = ativo;
    }

    // Construtor sem ID (para novos aparecimentos)
    public Aparecimento(Pokemon pokemon, double latitude, double longitude, Date dataHora, boolean ativo) {
        this.pokemon = pokemon;
        this.latitude = latitude;
        this.longitude = longitude;
        this.dataHora = dataHora;
        this.ativo = ativo;
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

    public Date getDataHora() {
        return dataHora;
    }

    public void setDataHora(Date dataHora) {
        this.dataHora = dataHora;
    }

    public boolean isAtivo() {
        return ativo;
    }

    public void setAtivo(boolean ativo) {
        this.ativo = ativo;
    }

    /**
     * Verifica se o aparecimento já expirou (Pokémon fica disponível por 15 minutos)
     * @return true se o aparecimento expirou
     */
    public boolean isExpirado() {
        if (!ativo) {
            return true;
        }
        
        Date agora = new Date();
        long diferencaMs = agora.getTime() - dataHora.getTime();
        long minutos = diferencaMs / (60 * 1000);
        
        return minutos > 15;
    }
    
    /**
     * Calcula a distância entre a localização do aparecimento e uma coordenada
     * @param lat Latitude da coordenada
     * @param lon Longitude da coordenada
     * @return Distância em metros
     */
    public float distanciaAte(double lat, double lon) {
        float[] results = new float[1];
        android.location.Location.distanceBetween(latitude, longitude, lat, lon, results);
        return results[0];
    }
}
