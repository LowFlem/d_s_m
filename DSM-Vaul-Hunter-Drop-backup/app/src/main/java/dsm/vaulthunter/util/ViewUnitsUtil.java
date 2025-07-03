package dsm.vaulthunter.util;

import android.content.res.Resources;
import android.util.DisplayMetrics;
import android.util.TypedValue;

/**
 * Utility class for handling view dimensions and conversions
 */
public class ViewUnitsUtil {
    /**
     * Convert device-independent pixels (dp) to pixels
     * @param dp Value in dp
     * @return Value in pixels
     */
    public static int convertDpToPixel(float dp) {
        DisplayMetrics metrics = Resources.getSystem().getDisplayMetrics();
        return (int) TypedValue.applyDimension(TypedValue.COMPLEX_UNIT_DIP, dp, metrics);
    }
    
    /**
     * Convert pixels to device-independent pixels (dp)
     * @param px Value in pixels
     * @return Value in dp
     */
    public static float convertPixelsToDp(float px) {
        DisplayMetrics metrics = Resources.getSystem().getDisplayMetrics();
        return px / metrics.density;
    }
    
    /**
     * Convert pixels to scaled pixels (sp)
     * @param px Value in pixels
     * @return Value in sp
     */
    public static float convertPixelsToSp(float px) {
        DisplayMetrics metrics = Resources.getSystem().getDisplayMetrics();
        return px / metrics.scaledDensity;
    }
    
    /**
     * Get screen width in pixels
     * @return Screen width in pixels
     */
    public static int getScreenWidthPixels() {
        return Resources.getSystem().getDisplayMetrics().widthPixels;
    }
    
    /**
     * Get screen height in pixels
     * @return Screen height in pixels
     */
    public static int getScreenHeightPixels() {
        return Resources.getSystem().getDisplayMetrics().heightPixels;
    }
    
    /**
     * Get the screen density
     * @return Screen density (e.g., 1.0 for mdpi, 1.5 for hdpi, etc.)
     */
    public static float getScreenDensity() {
        return Resources.getSystem().getDisplayMetrics().density;
    }
    
    /**
     * Calculate a size as a percentage of screen width
     * @param percentOfScreen Percentage of screen width (0-100)
     * @return Size in pixels
     */
    public static int getWidthPercentage(int percentOfScreen) {
        if (percentOfScreen < 0) percentOfScreen = 0;
        if (percentOfScreen > 100) percentOfScreen = 100;
        
        return (getScreenWidthPixels() * percentOfScreen) / 100;
    }
    
    /**
     * Calculate a size as a percentage of screen height
     * @param percentOfScreen Percentage of screen height (0-100)
     * @return Size in pixels
     */
    public static int getHeightPercentage(int percentOfScreen) {
        if (percentOfScreen < 0) percentOfScreen = 0;
        if (percentOfScreen > 100) percentOfScreen = 100;
        
        return (getScreenHeightPixels() * percentOfScreen) / 100;
    }
}