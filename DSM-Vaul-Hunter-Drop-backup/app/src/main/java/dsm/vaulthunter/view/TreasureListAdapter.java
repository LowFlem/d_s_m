package dsm.vaulthunter.view;

import android.content.Context;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.ImageView;
import android.widget.TextView;

import androidx.annotation.NonNull;
import androidx.recyclerview.widget.RecyclerView;

import java.util.List;

import dsm.vaulthunter.model.CollectedTreasure;
import teste.lucasvegi.pokemongooffline.R;

/**
 * Adapter for displaying lists of collected treasures in a RecyclerView
 */
public class TreasureListAdapter extends RecyclerView.Adapter<TreasureListAdapter.ViewHolder> {
    
    private final List<CollectedTreasure> treasures;
    private final Context context;
    private final OnTreasureClickListener listener;
    
    /**
     * Interface for handling treasure item clicks
     */
    public interface OnTreasureClickListener {
        void onTreasureClick(CollectedTreasure treasure);
    }
    
    /**
     * Constructor
     * @param context Application context
     * @param treasures List of treasures to display
     * @param listener Click listener
     */
    public TreasureListAdapter(Context context, List<CollectedTreasure> treasures, OnTreasureClickListener listener) {
        this.context = context;
        this.treasures = treasures;
        this.listener = listener;
    }
    
    @NonNull
    @Override
    public ViewHolder onCreateViewHolder(@NonNull ViewGroup parent, int viewType) {
        View view = LayoutInflater.from(parent.getContext())
                .inflate(R.layout.item_pokemon_troca, parent, false);
        return new ViewHolder(view);
    }
    
    @Override
    public void onBindViewHolder(@NonNull ViewHolder holder, int position) {
        final CollectedTreasure treasure = treasures.get(position);
        
        // Set treasure name
        holder.treasureNameText.setText(treasure.getTreasure().getName());
        
        // Set treasure stats
        holder.powerLevelText.setText(String.format("PL: %d", treasure.getPowerLevel()));
        holder.durabilityText.setText(String.format("DUR: %d", treasure.getDurability()));
        
        // Set treasure image
        int treasureImageResourceId = context.getResources().getIdentifier(
                "p" + treasure.getTreasure().getNumber(), "drawable", context.getPackageName());
        
        if (treasureImageResourceId != 0) {
            holder.treasureImage.setImageResource(treasureImageResourceId);
        } else {
            // Fallback to a default image
            holder.treasureImage.setImageResource(R.drawable.p0);
        }
        
        // Set type icons
        int primaryTypeId = treasure.getTreasure().getPrimaryType().getId();
        int primaryTypeIconId = context.getResources().getIdentifier(
                "tipo_" + primaryTypeId, "drawable", context.getPackageName());
        
        if (primaryTypeIconId != 0) {
            holder.primaryTypeIcon.setImageResource(primaryTypeIconId);
        }
        
        // Check for secondary type
        if (treasure.getTreasure().getSecondaryType() != null) {
            int secondaryTypeId = treasure.getTreasure().getSecondaryType().getId();
            int secondaryTypeIconId = context.getResources().getIdentifier(
                    "tipo_" + secondaryTypeId, "drawable", context.getPackageName());
            
            if (secondaryTypeIconId != 0) {
                holder.secondaryTypeIcon.setVisibility(View.VISIBLE);
                holder.secondaryTypeIcon.setImageResource(secondaryTypeIconId);
            } else {
                holder.secondaryTypeIcon.setVisibility(View.GONE);
            }
        } else {
            holder.secondaryTypeIcon.setVisibility(View.GONE);
        }
        
        // Set upgrade indicator if treasure can be upgraded
        holder.upgradeIndicator.setVisibility(
                treasure.getTreasure().canUpgrade() ? View.VISIBLE : View.GONE);
        
        // Set click listener
        holder.itemView.setOnClickListener(v -> {
            if (listener != null) {
                listener.onTreasureClick(treasure);
            }
        });
    }
    
    @Override
    public int getItemCount() {
        return treasures.size();
    }
    
    /**
     * Add a treasure to the list
     * @param treasure Treasure to add
     */
    public void addTreasure(CollectedTreasure treasure) {
        treasures.add(treasure);
        notifyItemInserted(treasures.size() - 1);
    }
    
    /**
     * Remove a treasure from the list
     * @param position Position to remove
     */
    public void removeTreasure(int position) {
        if (position >= 0 && position < treasures.size()) {
            treasures.remove(position);
            notifyItemRemoved(position);
        }
    }
    
    /**
     * Update a treasure in the list
     * @param treasure Updated treasure
     */
    public void updateTreasure(CollectedTreasure treasure) {
        for (int i = 0; i < treasures.size(); i++) {
            if (treasures.get(i).getId() == treasure.getId()) {
                treasures.set(i, treasure);
                notifyItemChanged(i);
                break;
            }
        }
    }
    
    /**
     * ViewHolder for treasure items
     */
    public static class ViewHolder extends RecyclerView.ViewHolder {
        final ImageView treasureImage;
        final TextView treasureNameText;
        final TextView powerLevelText;
        final TextView durabilityText;
        final ImageView primaryTypeIcon;
        final ImageView secondaryTypeIcon;
        final ImageView upgradeIndicator;
        
        public ViewHolder(@NonNull View itemView) {
            super(itemView);
            treasureImage = itemView.findViewById(R.id.imgPokemon);
            treasureNameText = itemView.findViewById(R.id.txtNomePokemon);
            powerLevelText = itemView.findViewById(R.id.txtCP);
            durabilityText = itemView.findViewById(R.id.txtHP);
            primaryTypeIcon = itemView.findViewById(R.id.imgTipo1);
            secondaryTypeIcon = itemView.findViewById(R.id.imgTipo2);
            upgradeIndicator = itemView.findViewById(R.id.imgEvolucao);
        }
    }
}