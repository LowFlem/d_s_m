package teste.lucasvegi.pokemongooffline.Model;

import android.graphics.Bitmap;

import com.google.android.gms.maps.model.BitmapDescriptor;
import com.google.android.gms.maps.model.BitmapDescriptorFactory;
import com.google.android.gms.maps.model.LatLng;
import com.google.android.gms.maps.model.MarkerOptions;

import java.io.Serializable;
import java.util.Date;

import teste.lucasvegi.pokemongooffline.R;

/**
 * Represents a PokéStop in the game world
 */
public class Pokestop implements Serializable {
    private static final long serialVersionUID = 1L;
    
    private String id;
    private String name;
    private transient Bitmap photo = null;
    private Double latitude;
    private Double longitude;
    private String description;
    private Date lastAccess;
    private boolean available;

    public Pokestop() {
    }

    public Pokestop(String id, String name) {
        this.id = id;
        this.name = name;
        this.lastAccess = null;
        this.available = true;
    }

    public String getID() {
        return id;
    }

    public void setID(String id) {
        this.id = id;
    }

    public String getNome() {
        return name;
    }

    public void setNome(String name) {
        this.name = name;
    }

    public String getDescri() {
        return description;
    }

    public void setDescri(String description) {
        this.description = description;
    }

    public double getlat() {
        return latitude;
    }

    public void setlat(double latitude) {
        this.latitude = latitude;
    }

    public double getlongi() {
        return longitude;
    }

    public void setlong(double longitude) {
        this.longitude = longitude;
    }

    public Bitmap getFoto() {
        return photo;
    }

    public void setFoto(Bitmap photo) {
        this.photo = photo;
    }

    public Date getUltimoAcesso() {
        return lastAccess;
    }

    public void setUltimoAcesso(Date time) {
        this.lastAccess = time;
    }

    public boolean getDisponivel() {
        return available;
    }

    public void setDisponivel(boolean available) {
        this.available = available;
    }

    /**
     * Gets the appropriate icon based on distance and availability
     */
    public BitmapDescriptor getIcon(boolean interactionPossible) {
        BitmapDescriptor bitmapDescriptor = null;

        if (interactionPossible && this.available) {
            bitmapDescriptor = BitmapDescriptorFactory
                    .fromResource(R.drawable.pokestop_perto);
        } else if (!interactionPossible && this.available) {
            bitmapDescriptor = BitmapDescriptorFactory
                    .fromResource(R.drawable.pokestop_longe);
        } else if (interactionPossible && !this.available) {
            bitmapDescriptor = BitmapDescriptorFactory
                    .fromResource(R.drawable.pokestop_perto_unable);
        } else if (!interactionPossible && !this.available) {
            bitmapDescriptor = BitmapDescriptorFactory
                    .fromResource(R.drawable.pokestop_longe_unable);
        }

        return bitmapDescriptor;
    }

    /**
     * Gets marker options for displaying on the map
     */
    public MarkerOptions getMarkerOptions(boolean interactionPossible) {
        MarkerOptions markeropt = new MarkerOptions();

        if (interactionPossible && this.available) {
            markeropt.icon(BitmapDescriptorFactory
                    .fromResource(R.drawable.pokestop_perto))
                    .position(new LatLng(latitude, longitude))
                    .title(name)
                    .alpha(3);
        } else if (!interactionPossible && this.available) {
            markeropt.icon(BitmapDescriptorFactory
                    .fromResource(R.drawable.pokestop_longe))
                    .position(new LatLng(latitude, longitude))
                    .title(name)
                    .alpha(3);
        } else if (interactionPossible && !this.available) {
            markeropt.icon(BitmapDescriptorFactory
                    .fromResource(R.drawable.pokestop_perto_unable))
                    .position(new LatLng(latitude, longitude))
                    .title(name)
                    .alpha(3);
        } else if (!interactionPossible && !this.available) {
            markeropt.icon(BitmapDescriptorFactory
                    .fromResource(R.drawable.pokestop_longe_unable))
                    .position(new LatLng(latitude, longitude))
                    .title(name)
                    .alpha(3);
        }
        return markeropt;
    }
}
