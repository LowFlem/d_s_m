package teste.lucasvegi.pokemongooffline.Model;

import android.content.ContentValues;
import android.database.Cursor;
import android.location.Location;
import android.util.Log;

import java.io.Serializable;

import teste.lucasvegi.pokemongooffline.Util.BancoDadosSingleton;

/**
 * Represents a Pokemon egg that can be hatched by walking
 */
public class Ovo implements Serializable {
    private static final long serialVersionUID = 1L;

    private int idEgg;
    private int idPokemon;
    private String idEggType;
    private int incubated;
    private int hatched;
    private int displayed;
    private int photo;
    private int incubatorPhoto;
    private double kilometers;
    private double kmWalked;
    private Location location = null;

    public Ovo(int idEgg, int idPokemon, String idEggType, int incubated, int hatched, int displayed, double kmWalked) {
        this.idEgg = idEgg;
        this.idPokemon = idPokemon;
        this.idEggType = idEggType;
        this.incubated = incubated;
        this.hatched = hatched;
        this.displayed = displayed;
        this.kmWalked = kmWalked;
    }

    public int getIdOvo() { 
        return idEgg; 
    }
    
    public int getidPokemon() { 
        return idPokemon; 
    }
    
    public String getIdTipoOvo() { 
        return idEggType; 
    }
    
    public int getIncubado() { 
        return incubated;
    }

    public int getFoto() {
        Cursor c = BancoDadosSingleton.getInstance().buscar(
                "egg e, eggtype t", 
                new String[]{"t.photo photo"}, 
                "e.idEggType = t.idEggType AND e.idEgg = '" + idEgg + "'", "");
        while (c.moveToNext()) {
            int idFoto = c.getColumnIndex("photo");
            photo = c.getInt(idFoto);
        }
        c.close();
        return photo;
    }

    public int getFotoIncubado() {
        Cursor c = BancoDadosSingleton.getInstance().buscar(
                "egg e, eggtype t", 
                new String[]{"t.incubatorPhoto incPhoto"}, 
                "e.idEggType = t.idEggType AND e.idEgg = '" + idEgg + "'", "");
        while (c.moveToNext()) {
            int idFotoInc = c.getColumnIndex("incPhoto");
            incubatorPhoto = c.getInt(idFotoInc);
        }
        c.close();
        return incubatorPhoto;
    }

    public double getKm() {
        Cursor c = BancoDadosSingleton.getInstance().buscar(
                "egg e, eggtype t", 
                new String[]{"t.kilometers km"}, 
                "e.idEggType = t.idEggType AND e.idEgg = '" + idEgg + "'", "");
        while (c.moveToNext()) {
            int idKm = c.getColumnIndex("km");
            kilometers = c.getDouble(idKm);
        }
        c.close();
        return kilometers;
    }

    public Location getLocalizacao() {
        return location;
    }
    
    public double getKmAndado() {
        return kmWalked;
    }
    
    public void setIdOvo(int idEgg) {
        this.idEgg = idEgg;
    }

    public void setidPokemon(int idPokemon) {
        this.idPokemon = idPokemon;
    }

    public void setIdTipoOvo(String idEggType) { 
        this.idEggType = idEggType; 
    }

    public void setIncubado(int inc) {
        ContentValues values = new ContentValues();
        values.put("incubated", inc);
        Log.i("EGGS", "idEgg: " + idEgg);
        BancoDadosSingleton.getInstance().atualizar("egg", values, "idEgg = '" + idEgg + "'");
        this.incubated = inc;
    }
    
    public void setLocalizacao(Location location) {
        this.location = location;
    }
    
    public void setKmAndado(double kmWalked) {
        this.kmWalked = kmWalked;
    }

    public int getChocado() {
        return hatched;
    }

    public void setChocado(int hatched) {
        this.hatched = hatched;
    }

    public int getExibido() {
        return displayed;
    }

    public void setExibido(int displayed) {
        this.displayed = displayed;
    }
}
