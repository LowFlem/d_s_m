package teste.lucasvegi.pokemongooffline.Controller;

import android.annotation.SuppressLint;
import android.content.Intent;
import android.os.Bundle;
import android.view.View;
import android.widget.Button;
import android.widget.TextView;

import androidx.appcompat.app.AppCompatActivity;

import java.text.SimpleDateFormat;
import java.util.Date;
import java.util.Locale;

import teste.lucasvegi.pokemongooffline.Model.ControladoraFachadaSingleton;
import teste.lucasvegi.pokemongooffline.Model.DSMClient;
import teste.lucasvegi.pokemongooffline.R;

/**
 * Main activity for the DSM-integrated Pokemon app
 */
public class DSMMainActivity extends AppCompatActivity {
    private static final String TAG = "DSMMainActivity";

    private TextView textEvent;
    private Button buttonTreasureHunt;

    @SuppressLint("SetTextI18n")
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_dsm_main);
        
        // Initialize the DSM client
        DSMClient.getInstance().connect("bootstrap.dsm.net:4001");
        
        // Initialize UI elements
        TextView textWelcome = findViewById(R.id.textWelcome);
        textEvent = findViewById(R.id.textEvent);
        buttonTreasureHunt = findViewById(R.id.buttonTreasureHunt);
        Button buttonTradePokemon = findViewById(R.id.buttonTradePokemon);
        
        // Set welcome message
        String username = ControladoraFachadaSingleton.getInstance().getUsuarioLogado().getNome();
        textWelcome.setText("Welcome, " + username + "!");
        
        // Update event info
        updateEventInfo();
    }
    
    @Override
    protected void onResume() {
        super.onResume();
        
        // Update event info
        updateEventInfo();
    }
    
    @SuppressLint("SetTextI18n")
    private void updateEventInfo() {
        // Get current active event
        DSMClient.RegionEvent currentEvent = DSMClient.getInstance().getCurrentActiveEvent();
        
        if (currentEvent != null) {
            // Format dates
            SimpleDateFormat dateFormat = new SimpleDateFormat("MMM dd, yyyy", Locale.US);
            String startDate = dateFormat.format(new Date(currentEvent.getStartTime()));
            String endDate = dateFormat.format(new Date(currentEvent.getEndTime()));
            
            // Update text
            String eventText = "Active Event: " + currentEvent.getRegionId() + "\n" +
                    "From: " + startDate + " to " + endDate;
            
            textEvent.setText(eventText);
            buttonTreasureHunt.setEnabled(true);
        } else {
            // No active event
            textEvent.setText("No active treasure hunt events");
            buttonTreasureHunt.setEnabled(false);
        }
    }
    
    public void onTreasureHuntClick(View view) {
        // Launch the Treasure Hunt activity
        Intent intent = new Intent(this, TreasureHuntActivity.class);
        startActivity(intent);
    }
    
    public void onTradePokemonClick(View view) {
        // Launch the Trade Pokemon activity
        Intent intent = new Intent(this, TrocaListaPokemonActivity.class);
        startActivity(intent);
    }
}
