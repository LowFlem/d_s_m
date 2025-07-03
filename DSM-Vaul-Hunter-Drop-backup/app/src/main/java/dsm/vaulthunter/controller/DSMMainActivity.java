package dsm.vaulthunter.controller;

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

import dsm.vaulthunter.model.AppController;
import dsm.vaulthunter.model.DSMClient;
import teste.lucasvegi.pokemongooffline.R;

/**
 * Main activity for the DSM-integrated Vault Hunter game
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
        DSMClient.getInstance().connect("bootstrap.dsm.network:4001");
        
        // Initialize UI elements
        TextView textWelcome = findViewById(R.id.textWelcome);
        textEvent = findViewById(R.id.textEvent);
        buttonTreasureHunt = findViewById(R.id.buttonTreasureHunt);
        Button buttonTradeTreasures = findViewById(R.id.buttonTradePokemon);
        
        // Set welcome message
        String username = AppController.getInstance().getLoggedInUser().getName();
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
            String eventText = "Active Event: " + currentEvent.getName() + "\n" +
                    "Region: " + currentEvent.getRegionId() + "\n" +
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
    
    public void onTradeTreasuresClick(View view) {
        // This would launch the trading activity
        // For now, just show the main map
        Intent intent = new Intent(this, MapActivity.class);
        startActivity(intent);
    }
}