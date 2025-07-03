package dsm.vaulthunter.view;

import android.content.Context;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.CheckBox;
import android.widget.ImageView;
import android.widget.TextView;

import androidx.annotation.NonNull;
import androidx.recyclerview.widget.RecyclerView;

import java.util.HashSet;
import java.util.List;
import java.util.Set;

import dsm.vaulthunter.model.CollectedTreasure;
import teste.lucasvegi.pokemongooffline.R;

/**
 * Adapter for displaying treasures available for trading
 */
public class TradeAdapter extends RecyclerView.Adapter<TradeAdapter.ViewHolder> {
    
    private final List<CollectedTreasure> treasures;
    private final Context context;
    private final Set<CollectedTreasure> selectedTreasures;
    private final OnSelectionChangedListener listener;
    
    /**
     * Interface for selection change callbacks
     */
    public interface OnSelectionChangedListener {
        void onSelectionChanged(int count);
    }
    
    /**
     * Constructor
     * @param context Application context
     * @param treasures List of collected treasures
     * @param listener Selection change listener
     */
    public TradeAdapter(Context context, List<CollectedTreasure> treasures, OnSelectionChangedListener listener) {
        this.context = context;
        this.treasures = treasures;
        this.selectedTreasures = new HashSet<>();
        this.listener = listener;
    }
    
    @NonNull
    @Override
    public ViewHolder onCreateViewHolder(@NonNull ViewGroup parent, int viewType) {
        View view = LayoutInflater.from(parent.getContext())
                .inflate(R.layout.item_trade, parent, false);
        return new ViewHolder(view);
    }
    
    @Override
    public void onBindViewHolder(@NonNull ViewHolder holder, int position) {
        final CollectedTreasure collectedTreasure = treasures.get(position);
        
        // Set treasure details
        holder.treasureNameText.setText(collectedTreasure.getTreasure().getName());
        holder.powerLevelText.setText(String.format("PL: %d", collectedTreasure.getPowerLevel()));
        holder.durabilityText.setText(String.format("DUR: %d", collectedTreasure.getDurability()));
        
        // Set treasure image
        int treasureImageResourceId = context.getResources().getIdentifier(
                "p" + collectedTreasure.getTreasure().getNumber(), "drawable", context.getPackageName());
        
        if (treasureImageResourceId != 0) {
            holder.treasureImage.setImageResource(treasureImageResourceId);
        } else {
            // Fallback to a default image
            holder.treasureImage.setImageResource(R.drawable.p0);
        }
        
        // Set type icons
        int typeIconId = context.getResources().getIdentifier(
                "tipo_" + collectedTreasure.getTreasure().getPrimaryType().getId(), 
                "drawable", 
                context.getPackageName());
        
        if (typeIconId != 0) {
            holder.typeIcon.setImageResource(typeIconId);
        }
        
        // Set selection state
        holder.selectCheckbox.setChecked(selectedTreasures.contains(collectedTreasure));
        
        // Handle selection changes
        holder.selectCheckbox.setOnCheckedChangeListener((buttonView, isChecked) -> {
            if (isChecked) {
                selectedTreasures.add(collectedTreasure);
            } else {
                selectedTreasures.remove(collectedTreasure);
            }
            
            // Notify listener of selection change
            if (listener != null) {
                listener.onSelectionChanged(selectedTreasures.size());
            }
        });
        
        // Make the entire row clickable to toggle selection
        holder.itemView.setOnClickListener(v -> {
            boolean newState = !holder.selectCheckbox.isChecked();
            holder.selectCheckbox.setChecked(newState);
        });
    }
    
    @Override
    public int getItemCount() {
        return treasures.size();
    }
    
    /**
     * Get the set of selected treasures
     * @return Set of selected treasures
     */
    public Set<CollectedTreasure> getSelectedTreasures() {
        return selectedTreasures;
    }
    
    /**
     * Clear all selections
     */
    public void clearSelections() {
        selectedTreasures.clear();
        notifyDataSetChanged();
        
        if (listener != null) {
            listener.onSelectionChanged(0);
        }
    }
    
    /**
     * ViewHolder for treasure trade items
     */
    public static class ViewHolder extends RecyclerView.ViewHolder {
        final ImageView treasureImage;
        final TextView treasureNameText;
        final TextView powerLevelText;
        final TextView durabilityText;
        final ImageView typeIcon;
        final CheckBox selectCheckbox;
        
        public ViewHolder(@NonNull View itemView) {
            super(itemView);
            treasureImage = itemView.findViewById(R.id.imgPokemonTroca);
            treasureNameText = itemView.findViewById(R.id.txtNomePokemonTroca);
            powerLevelText = itemView.findViewById(R.id.txtCPPokemonTroca);
            durabilityText = itemView.findViewById(R.id.txtHPPokemonTroca);
            typeIcon = itemView.findViewById(R.id.imgTipoPokemonTroca);
            selectCheckbox = itemView.findViewById(R.id.checkBoxTroca);
        }
    }
}