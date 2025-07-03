package dsm.vaulthunter.view;

import android.content.Context;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.ImageView;
import android.widget.ProgressBar;
import android.widget.TextView;

import androidx.annotation.NonNull;
import androidx.recyclerview.widget.RecyclerView;

import java.util.List;

import dsm.vaulthunter.model.Vault;
import teste.lucasvegi.pokemongooffline.R;

/**
 * Adapter for displaying vaults in the vault screen
 */
public class VaultAdapter extends RecyclerView.Adapter<VaultAdapter.ViewHolder> {
    
    private final List<Vault> vaults;
    private final Context context;
    private final OnVaultClickListener listener;
    
    /**
     * Interface for handling vault item clicks
     */
    public interface OnVaultClickListener {
        void onVaultClick(Vault vault);
    }
    
    /**
     * Constructor
     * @param context Application context
     * @param vaults List of vaults to display
     * @param listener Click listener
     */
    public VaultAdapter(Context context, List<Vault> vaults, OnVaultClickListener listener) {
        this.context = context;
        this.vaults = vaults;
        this.listener = listener;
    }
    
    @NonNull
    @Override
    public ViewHolder onCreateViewHolder(@NonNull ViewGroup parent, int viewType) {
        View view = LayoutInflater.from(parent.getContext())
                .inflate(R.layout.item_vault, parent, false);
        return new ViewHolder(view);
    }
    
    @Override
    public void onBindViewHolder(@NonNull ViewHolder holder, int position) {
        final Vault vault = vaults.get(position);
        
        // Set vault name/title based on variant
        String vaultTitle = vault.getVariant().name() + " Vault";
        holder.vaultTitle.setText(vaultTitle);
        
        // Set token amount
        holder.tokenAmount.setText(String.format("%,d DSM", vault.getTokenAmount()));
        
        // Set vault status
        String statusText;
        switch (vault.getStatus()) {
            case MINTED:
                statusText = "Available";
                break;
            case CLAIMED:
                statusText = "Claimed";
                break;
            case WITHDRAWN:
                statusText = "Withdrawn";
                break;
            default:
                statusText = "Unknown";
        }
        holder.vaultStatus.setText(statusText);
        
        // Set vault image based on variant
        int vaultImageResourceId = context.getResources().getIdentifier(
                vault.getVaultDrawableResource(), "drawable", context.getPackageName());
        
        if (vaultImageResourceId != 0) {
            holder.vaultImage.setImageResource(vaultImageResourceId);
        } else {
            // Fallback to a default image - using bronze chest for now
            holder.vaultImage.setImageResource(R.drawable.dsm_bronze_chest);
        }
        
        // Set progress visibility based on status
        if (vault.getStatus() == Vault.VaultStatus.CLAIMED) {
            holder.progressBar.setVisibility(View.VISIBLE);
        } else {
            holder.progressBar.setVisibility(View.GONE);
        }
        
        // Set click listener
        holder.itemView.setOnClickListener(v -> {
            if (listener != null) {
                listener.onVaultClick(vault);
            }
        });
    }
    
    @Override
    public int getItemCount() {
        return vaults.size();
    }
    
    /**
     * Remove a vault from the list
     * @param vault Vault to remove
     */
    public void removeVault(Vault vault) {
        int position = vaults.indexOf(vault);
        if (position >= 0) {
            vaults.remove(position);
            notifyItemRemoved(position);
        }
    }
    
    /**
     * Add a vault to the list
     * @param vault Vault to add
     */
    public void addVault(Vault vault) {
        vaults.add(vault);
        notifyItemInserted(vaults.size() - 1);
    }
    
    /**
     * Update a vault in the list
     * @param vault Vault to update
     */
    public void updateVault(Vault vault) {
        int position = -1;
        
        // Find the vault by ID
        for (int i = 0; i < vaults.size(); i++) {
            if (vaults.get(i).getVaultId().equals(vault.getVaultId())) {
                position = i;
                break;
            }
        }
        
        if (position >= 0) {
            vaults.set(position, vault);
            notifyItemChanged(position);
        }
    }
    
    /**
     * ViewHolder for vault items
     */
    public static class ViewHolder extends RecyclerView.ViewHolder {
        final ImageView vaultImage;
        final TextView vaultTitle;
        final TextView tokenAmount;
        final TextView vaultStatus;
        final ProgressBar progressBar;
        
        public ViewHolder(@NonNull View itemView) {
            super(itemView);
            vaultImage = itemView.findViewById(R.id.vault_image);
            vaultTitle = itemView.findViewById(R.id.vault_title);
            tokenAmount = itemView.findViewById(R.id.token_amount);
            vaultStatus = itemView.findViewById(R.id.vault_status);
            progressBar = itemView.findViewById(R.id.progressBar);
        }
    }
}