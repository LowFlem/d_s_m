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
import dsm.vaulthunter.model.Treasure;
import dsm.vaulthunter.model.User;
import teste.lucasvegi.pokemongooffline.R;

/**
 * Adapter for displaying treasures in the TreasureDex
 */
public class TreasureDexAdapter extends RecyclerView.Adapter<TreasureDexAdapter.ViewHolder> {
    
    private final List<Treasure> treasures;
    private final User user;
    private final Context context;
    private final OnTreasureClickListener listener;
    
    /**
     * Interface for handling treasure item clicks
     */
    public interface OnTreasureClickListener {
        void onTreasureClick(Treasure treasure);
    }
    
    /**
     * Constructor
     * @param context Application context
     * @param treasures List of treasures to display
     * @param user Current user
     * @param listener Click listener
     */
    public TreasureDexAdapter(Context context, List<Treasure> treasures, User user, OnTreasureClickListener listener) {
        this.context = context;
        this.treasures = treasures;
        this.user = user;
        this.listener = listener;
    }
    
    @NonNull
    @Override
    public ViewHolder onCreateViewHolder(@NonNull ViewGroup parent, int viewType) {
        View view = LayoutInflater.from(parent.getContext())
                .inflate(R.layout.item_pokedex, parent, false);
        return new ViewHolder(view);
    }
    
    @Override
    public void onBindViewHolder(@NonNull ViewHolder holder, int position) {
        final Treasure treasure = treasures.get(position);
        
        // Set treasure number and name
        holder.treasureNumberText.setText(String.format("#%03d", treasure.getNumber()));
        holder.treasureNameText.setText(treasure.getName());
        
        // Set treasure image
        int treasureImageResourceId = context.getResources().getIdentifier(
                "p" + treasure.getNumber(), "drawable", context.getPackageName());
        
        if (treasureImageResourceId != 0) {
            holder.treasureImage.setImageResource(treasureImageResourceId);
        } else {
            // Fallback to a default image
            holder.treasureImage.setImageResource(R.drawable.p0);
        }
        
        // Check if user has this treasure
        int captureCount = user.getTreasureCount(treasure);
        if (captureCount > 0) {
            // Treasure has been collected
            holder.treasureCountText.setText(String.valueOf(captureCount));
            holder.treasureCountText.setVisibility(View.VISIBLE);
            
            // Display treasure normally
            holder.treasureImage.setAlpha(1.0f);
            holder.treasureNameText.setAlpha(1.0f);
        } else {
            // Treasure has not been collected
            holder.treasureCountText.setVisibility(View.INVISIBLE);
            
            // Display treasure silhouette
            holder.treasureImage.setAlpha(0.4f);
            holder.treasureNameText.setAlpha(0.4f);
        }
        
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
     * ViewHolder for treasure items
     */
    public static class ViewHolder extends RecyclerView.ViewHolder {
        final ImageView treasureImage;
        final TextView treasureNumberText;
        final TextView treasureNameText;
        final TextView treasureCountText;
        
        public ViewHolder(@NonNull View itemView) {
            super(itemView);
            treasureImage = itemView.findViewById(R.id.imgPokemonPokedex);
            treasureNumberText = itemView.findViewById(R.id.txtNumeroPokedex);
            treasureNameText = itemView.findViewById(R.id.txtNomePokedex);
            treasureCountText = itemView.findViewById(R.id.txtQtdePokedex);
        }
    }
}