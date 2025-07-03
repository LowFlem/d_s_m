package teste.lucasvegi.pokemongooffline.Controller;

import android.annotation.SuppressLint;
import android.app.Activity;
import android.app.ActivityOptions;
import android.content.ContentValues;
import android.content.Context;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.database.Cursor;
import android.graphics.Color;
import android.location.Criteria;
import android.location.Location;
import android.location.LocationListener;
import android.location.LocationManager;
import android.media.session.MediaSessionManager;
import android.os.Bundle;
import android.util.Log;
import android.view.View;
import android.widget.Button;
import android.widget.ImageView;
import android.widget.RelativeLayout;
import android.widget.TextView;
import android.widget.Toast;

import androidx.annotation.NonNull;

import com.google.android.gms.maps.GoogleMap;
import com.google.android.gms.maps.SupportMapFragment;
import com.google.android.gms.maps.model.LatLng;

import teste.lucasvegi.pokemongooffline.Model.Aparecimento;
import teste.lucasvegi.pokemongooffline.Model.ControladoraFachadaSingleton;
import teste.lucasvegi.pokemongooffline.Model.Pokemon;
import teste.lucasvegi.pokemongooffline.Model.PokemonCapturado;
import teste.lucasvegi.pokemongooffline.R;
import teste.lucasvegi.pokemongooffline.Util.BancoDadosSingleton;
import teste.lucasvegi.pokemongooffline.View.AdapterPokedex;

public class DetalhesPokedexActivity extends Activity implements LocationListener {

    private Pokemon pkmn;
    private Pokemon pkmnEvolution;
    public LocationManager lm;
    public Criteria criteria;
    public String provider;
    public int TIME = 5000;
    public int DIST = 0;
    public Location current;

    private int candiesRequired;
    private int candiesObtained;

    // Helper constant for the setResult() method
    private final int UPDATE_SCREEN = 1;

    public void configureLocationCriteria() {
        lm = (LocationManager) getSystemService(Context.LOCATION_SERVICE);
        criteria = new Criteria();

        PackageManager packageManager = getPackageManager();
        boolean hasGPS = packageManager.hasSystemFeature(PackageManager.FEATURE_LOCATION_GPS);

        if (hasGPS) {
            criteria.setAccuracy(Criteria.ACCURACY_FINE);
            Log.i("LOCATION", "using GPS");
        } else {
            criteria.setAccuracy(Criteria.ACCURACY_COARSE);
            Log.i("LOCATION", "using WiFi or data");
        }
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {

    }

    private void setTextViewBackground(TextView txtType1, String nome) {
    }

    @SuppressLint("MissingPermission")
    @Override
    protected void onStart() {
        super.onStart();
        Log.i("PROVIDER", "start");

        provider = lm.getBestProvider(criteria, true);

        if (provider == null) {
            Log.e("PROVIDER", "No provider found");
        } else {
            Log.i("PROVIDER", "Currently using provider " + provider);

            lm.requestLocationUpdates(provider, TIME, DIST, this);
        }
    }

    public void clickBackDetail(View v){
        finish();
    }

    public void clickLocations(View v){
        //Toast.makeText(this, "Locations of " + pkmn.getNome(), Toast.LENGTH_SHORT).show();

        Intent it = new Intent(this,MapCapturasActivity.class);
        it.putExtra("pkmn",pkmn);
        startActivity(it);
    }

    private void evolve(){
        Aparecimento ap = new Aparecimento();
        ap.setLatitude(current.getLatitude());
        ap.setLongitude(current.getLongitude());
        ap.setPokemon(pkmnEvolution);
        Log.i("EVOLUTION", "EVOLUTION NAME: " + ap.getPokemon().getNome());

        // Send capture to the server before closing screen.
        ControladoraFachadaSingleton.getInstance().getUsuario().capturar(ap);
        Log.i("EVOLUTION","Evolution captured");

        // Subtract the candies used in evolution
        ControladoraFachadaSingleton.getInstance().getUsuario().somarDoces(pkmn, -candiesRequired-3);
        Log.i("EVOLUTION","Pokemon evolved");

        // Display success message on screen
        Toast.makeText(getBaseContext(),pkmn.getNome() + " has evolved! \\o/",Toast.LENGTH_LONG).show();

        // Starting activity for the evolved pokemon
        Intent it = new Intent(this, DetalhesPokedexActivity.class);
        it.putExtra("pkmn", ap.getPokemon());
        startActivity(it);

        // Sending request to update the Pokedex screen at the end of this activity
        Intent itResult = new Intent();
        setResult(UPDATE_SCREEN,itResult);

        // Ending activity
        finish();
    }

    public void clickEvolve(View v){
        int remaining = candiesRequired-candiesObtained;

        // Checking if the pokemon has an evolution
        if(pkmnEvolution == null){
            Toast.makeText(this,"This Pokemon has no evolution!",Toast.LENGTH_LONG).show();
        }

        // Checking candy quantity
        else if(candiesRequired > candiesObtained){
            Toast.makeText(this,"You need "+ remaining +" more candies to evolve!",Toast.LENGTH_LONG).show();
        }

        // Evolving pokemon if there is one available
        else if (pkmn.estaDisponivel(true)){ // If this happens, we've already updated the 'isBlocked' flag in the pokemonuser table in the Database
            evolve();
            ControladoraFachadaSingleton.getInstance().aumentaXp("evolves");   //update user XP after evolving a Pokemon
            PokemonCapturado toEdit = null;
            for (PokemonCapturado capt: ControladoraFachadaSingleton.getInstance().getUsuario().getPokemons().get(pkmn) ) {
                if(capt.getEstaBloqueado() == 0) {
                    capt.setEstaBloqueado(1);
                    toEdit = capt;
                    break;
                }
            }
        }

        // If no pokemon is available, we inform this via Toast
        else{
            Toast.makeText(getBaseContext(),"There are no Pokemon named " + pkmn.getNome() + " available for evolution!",Toast.LENGTH_LONG).show();
        }
    }

    @Override
    public void onLocationChanged(@NonNull Location location) {

    }
}