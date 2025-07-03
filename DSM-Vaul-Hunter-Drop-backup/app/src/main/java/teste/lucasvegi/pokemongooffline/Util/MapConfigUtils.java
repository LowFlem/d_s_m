package teste.lucasvegi.pokemongooffline.Util;

import android.content.Context;
import android.os.Environment;
import android.preference.PreferenceManager;

import org.osmdroid.config.Configuration;

import java.io.File;

/**
 * Utility class for OpenStreetMap configuration
 */
public class MapConfigUtils {

    /**
     * Initializes the OSMDroid configuration with necessary settings
     * 
     * @param context Application context
     */
    public static void initializeOsmDroid(Context context) {
        // Set up the osmdroid configuration
        Configuration.getInstance().load(context, PreferenceManager.getDefaultSharedPreferences(context));
        
        // Set the user agent to prevent 403 errors with OSM servers
        Configuration.getInstance().setUserAgentValue(context.getPackageName());
        
        // Set up the osmdroid cache directory
        File osmCacheDir = new File(context.getCacheDir(), "osmdroid");
        if (!osmCacheDir.exists()) {
            osmCacheDir.mkdirs();
        }
        Configuration.getInstance().setOsmdroidBasePath(osmCacheDir);
        
        File tileCache = new File(osmCacheDir, "tiles");
        if (!tileCache.exists()) {
            tileCache.mkdirs();
        }
        Configuration.getInstance().setOsmdroidTileCache(tileCache);
    }
}