package dsm.vaulthunter.controller;

import android.content.Intent;
import android.os.Bundle;
import android.view.View;
import android.widget.SearchView;
import android.widget.Toast;

import androidx.appcompat.app.AppCompatActivity;
import androidx.recyclerview.widget.GridLayoutManager;
import androidx.recyclerview.widget.RecyclerView;

import java.util.ArrayList;
import java.util.Collections;
import java.util.Comparator;
import java.util.List;

import dsm.vaulthunter.model.AppController;
import dsm.vaulthunter.model.Treasure;
import dsm.vaulthunter.model.User;
import dsm.vaulthunter.view.TreasureDexAdapter;
import teste.lucasvegi.pokemongooffline.R;

/**
 * Activity to display the TreasureDex (catalog of treasures)
 */
public class TreasureDexActivity extends AppCompatActivity implements TreasureDexAdapter.OnTreasureClickListener {
    private static final String TAG = "TreasureDexActivity";
    
    private RecyclerView treasureDexRecyclerView;
    private SearchView searchView;
    private TreasureDexAdapter adapter;
    
    private List<Treasure> allTreasures;
    private List<Treasure> filteredTreasures;
    
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_pokedex);
        
        // Initialize UI elements
        treasureDexRecyclerView = findViewById(R.id.recicladorPokedex);
        searchView = findViewById(R.id.searchView);
        
        // Get all treasures from controller
        allTreasures = AppController.getInstance().getAllTreasures();
        
        // Sort treasures by number
        Collections.sort(allTreasures, Comparator.comparingInt(Treasure::getNumber));
        
        // Initialize filtered list
        filteredTreasures = new ArrayList<>(allTreasures);
        
        // Set up recycler view
        setupRecyclerView();
        
        // Set up search
        setupSearch();
    }
    
    private void setupRecyclerView() {
        // Create grid layout manager (3 columns)
        GridLayoutManager layoutManager = new GridLayoutManager(this, 3);
        treasureDexRecyclerView.setLayoutManager(layoutManager);
        
        // Get current user
        User user = AppController.getInstance().getLoggedInUser();
        
        // Create and set adapter
        adapter = new TreasureDexAdapter(this, filteredTreasures, user, this);
        treasureDexRecyclerView.setAdapter(adapter);
    }
    
    private void setupSearch() {
        searchView.setOnQueryTextListener(new SearchView.OnQueryTextListener() {
            @Override
            public boolean onQueryTextSubmit(String query) {
                filterTreasures(query);
                return true;
            }
            
            @Override
            public boolean onQueryTextChange(String newText) {
                filterTreasures(newText);
                return true;
            }
        });
        
        // Add clear listener
        searchView.setOnCloseListener(() -> {
            filterTreasures("");
            return false;
        });
    }
    
    private void filterTreasures(String query) {
        filteredTreasures.clear();
        
        if (query.isEmpty()) {
            // No filter, show all treasures
            filteredTreasures.addAll(allTreasures);
        } else {
            // Filter by name or number
            String lowerQuery = query.toLowerCase();
            
            for (Treasure treasure : allTreasures) {
                if (treasure.getName().toLowerCase().contains(lowerQuery) ||
                    String.valueOf(treasure.getNumber()).contains(lowerQuery)) {
                    filteredTreasures.add(treasure);
                }
            }
        }
        
        // Update the adapter
        adapter.notifyDataSetChanged();
        
        // Show message if no treasures match
        if (filteredTreasures.isEmpty()) {
            Toast.makeText(this, R.string.no_treasures_found, Toast.LENGTH_SHORT).show();
        }
    }
    
    @Override
    public void onTreasureClick(Treasure treasure) {
        // Open treasure details activity
        Intent intent = new Intent(this, TreasureDetailsActivity.class);
        intent.putExtra("treasure_number", treasure.getNumber());
        startActivity(intent);
    }
    
    // Click handlers for buttons
    public void goMapa(View view) {
        finish();
    }
}