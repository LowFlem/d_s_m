package dsm.vaulthunter.controller;

import android.os.Bundle;
import android.view.View;
import android.widget.Button;
import android.widget.TextView;
import android.widget.Toast;

import androidx.appcompat.app.AppCompatActivity;
import androidx.recyclerview.widget.LinearLayoutManager;
import androidx.recyclerview.widget.RecyclerView;

import java.util.ArrayList;
import java.util.List;

import dsm.vaulthunter.model.AppController;
import dsm.vaulthunter.model.DSMClient;
import dsm.vaulthunter.model.Vault;
import teste.lucasvegi.pokemongooffline.R;

/**
 * Activity for DSM treasure hunt events that provide special vault drops
 */
public class TreasureHuntActivity extends AppCompatActivity {
    private static final String TAG = "TreasureHuntActivity";
    
    private TextView eventTitleText;
    private TextView eventDescriptionText;
    private TextView tokenBalanceText;
    private Button claimButton;
    private Button withdrawButton;
    private RecyclerView vaultsRecyclerView;
    
    // Track claimed vaults from this event
    private List<Vault> claimedVaults = new ArrayList<>();
    private long tokenBalance = 0;
    
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_treasure_hunt);
        
        // Initialize UI elements
        initializeViews();
        
        // Get current event information
        DSMClient.RegionEvent currentEvent = DSMClient.getInstance().getCurrentActiveEvent();
        
        if (currentEvent != null) {
            // Set event information
            eventTitleText.setText(currentEvent.getName());
            eventDescriptionText.setText("Find and claim special vault drops in the " + 
                    currentEvent.getRegionId() + " region!");
            
            // Create some sample claimed vaults for demo purposes
            createSampleClaimedVaults(currentEvent.getRegionId());
            
            // Update token balance
            updateTokenBalance();
            
            // Set up RecyclerView - in a real implementation, this would use an adapter
            // vaultsRecyclerView.setAdapter(new VaultsAdapter(claimedVaults));
            // vaultsRecyclerView.setLayoutManager(new LinearLayoutManager(this));
            
            // Update button states
            updateButtonStates();
        } else {
            // No active event - shouldn't happen normally since the button is disabled in DSMMainActivity
            Toast.makeText(this, "No active event found", Toast.LENGTH_SHORT).show();
            finish();
        }
    }
    
    private void initializeViews() {
        eventTitleText = findViewById(R.id.event_title);
        eventDescriptionText = findViewById(R.id.event_description);
        tokenBalanceText = findViewById(R.id.token_balance);
        claimButton = findViewById(R.id.claim_button);
        withdrawButton = findViewById(R.id.withdraw_button);
        vaultsRecyclerView = findViewById(R.id.vaults_recycler_view);
        
        // Set up click listeners
        claimButton.setOnClickListener(this::onClaimClick);
        withdrawButton.setOnClickListener(this::onWithdrawClick);
        
        // Also need a back button
        findViewById(R.id.back_button).setOnClickListener(this::onBackClick);
    }
    
    private void createSampleClaimedVaults(String regionId) {
        // Add some sample claimed vaults for demonstration
        Vault vault1 = new Vault("VAULT-SAMPLE1", regionId, Vault.VaultVariant.BRONZE, 
                40.7128, -74.0060); // Sample coordinates (NYC)
        vault1.claim(AppController.getInstance().getLoggedInUser().getId() + "", System.currentTimeMillis());
        claimedVaults.add(vault1);
        
        Vault vault2 = new Vault("VAULT-SAMPLE2", regionId, Vault.VaultVariant.SILVER, 
                51.5074, -0.1278); // Sample coordinates (London)
        vault2.claim(AppController.getInstance().getLoggedInUser().getId() + "", System.currentTimeMillis());
        claimedVaults.add(vault2);
    }
    
    private void updateTokenBalance() {
        // Calculate total tokens from claimed vaults
        tokenBalance = 0;
        for (Vault vault : claimedVaults) {
            if (vault.getStatus() == Vault.VaultStatus.CLAIMED) {
                tokenBalance += vault.getTokenAmount();
            }
        }
        
        // Update UI
        tokenBalanceText.setText(String.format("Token Balance: %,d", tokenBalance));
    }
    
    private void updateButtonStates() {
        // Claim button is enabled when there are active vaults in the world
        claimButton.setEnabled(true);
        
        // Withdraw button is enabled when there are claimed vaults
        withdrawButton.setEnabled(!claimedVaults.isEmpty());
    }
    
    public void onClaimClick(View view) {
        // In a real implementation, this would open the map to find vaults
        // For now, just show a toast
        Toast.makeText(this, "Opening map to find vaults...", Toast.LENGTH_SHORT).show();
        
        // Add another sample vault for demonstration
        Vault newVault = new Vault("VAULT-SAMPLE-" + (claimedVaults.size() + 1),
                DSMClient.getInstance().getCurrentActiveEvent().getRegionId(),
                Vault.VaultVariant.BRONZE,
                35.6762, 139.6503); // Sample coordinates (Tokyo)
        
        newVault.claim(AppController.getInstance().getLoggedInUser().getId() + "", System.currentTimeMillis());
        claimedVaults.add(newVault);
        
        updateTokenBalance();
        updateButtonStates();
    }
    
    public void onWithdrawClick(View view) {
        // "Withdraw" tokens from vaults - in a real implementation, this would interact with the blockchain
        int withdrawnCount = 0;
        
        for (Vault vault : claimedVaults) {
            if (vault.getStatus() == Vault.VaultStatus.CLAIMED) {
                vault.withdraw();
                withdrawnCount++;
            }
        }
        
        if (withdrawnCount > 0) {
            Toast.makeText(this, 
                    String.format("Withdrew %,d tokens from %d vaults!", tokenBalance, withdrawnCount), 
                    Toast.LENGTH_LONG).show();
            
            // Reset token balance and update UI
            tokenBalance = 0;
            tokenBalanceText.setText("Token Balance: 0");
            
            // Add XP for withdrawing vaults
            boolean leveledUp = AppController.getInstance().addExperiencePoints("vault");
            if (leveledUp) {
                Toast.makeText(this, "Level up! You are now level " + 
                        AppController.getInstance().getLoggedInUser().getLevel(), 
                        Toast.LENGTH_LONG).show();
            }
        } else {
            Toast.makeText(this, "No vaults to withdraw from", Toast.LENGTH_SHORT).show();
        }
    }
    
    public void onBackClick(View view) {
        finish();
    }
}