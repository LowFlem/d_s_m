package teste.lucasvegi.pokemongooffline.Model;

import java.io.Serializable;
import java.util.Date;

/**
 * Represents an interaction between a user and a PokéStop
 */
public class InteracaoPokestop implements Serializable {
    private static final long serialVersionUID = 1L;
    
    private Pokestop pokestop;
    private Usuario user;
    private Date lastAccess;

    public InteracaoPokestop(Pokestop pokestop, Usuario user, Date lastAccess) {
        this.pokestop = pokestop;
        this.user = user;
        this.lastAccess = lastAccess;
    }

    public Pokestop getPokestop() {
        return pokestop;
    }

    public void setPokestop(Pokestop pokestop) {
        this.pokestop = pokestop;
    }

    public Usuario getUser() {
        return user;
    }

    public void setUser(Usuario user) {
        this.user = user;
    }

    public Date getUltimoAcesso() {
        return lastAccess;
    }

    public void setUltimoAcesso(Date lastAccess) {
        this.lastAccess = lastAccess;
    }
}
