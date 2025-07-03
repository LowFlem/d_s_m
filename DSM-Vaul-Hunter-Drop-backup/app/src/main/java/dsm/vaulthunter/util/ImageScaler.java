package dsm.vaulthunter.util;

import android.content.Context;
import android.graphics.Bitmap;
import android.graphics.BitmapFactory;
import android.graphics.Matrix;
import android.graphics.drawable.BitmapDrawable;
import android.graphics.drawable.Drawable;
import android.util.Log;
import android.widget.ImageView;

import java.io.File;
import java.io.FileInputStream;
import java.io.IOException;
import java.io.InputStream;

/**
 * Utility class for handling image scaling and loading
 * Automatically scales images to appropriate sizes for different view types
 */
public class ImageScaler {
    private static final String TAG = "ImageScaler";
    
    /**
     * Load and scale an image from resources
     * @param context Application context
     * @param resourceId Resource ID of the image
     * @param targetWidth Target width (use 0 for original size)
     * @param targetHeight Target height (use 0 for original size)
     * @return Scaled bitmap
     */
    public static Bitmap loadAndScaleResource(Context context, int resourceId, int targetWidth, int targetHeight) {
        try {
            // If target dimensions are 0, use original size
            if (targetWidth == 0 || targetHeight == 0) {
                return BitmapFactory.decodeResource(context.getResources(), resourceId);
            }
            
            // First decode with inJustDecodeBounds=true to check dimensions
            final BitmapFactory.Options options = new BitmapFactory.Options();
            options.inJustDecodeBounds = true;
            BitmapFactory.decodeResource(context.getResources(), resourceId, options);
            
            // Calculate inSampleSize
            options.inSampleSize = calculateInSampleSize(options, targetWidth, targetHeight);
            
            // Decode bitmap with inSampleSize set
            options.inJustDecodeBounds = false;
            Bitmap scaledBitmap = BitmapFactory.decodeResource(context.getResources(), resourceId, options);
            
            return getResizedBitmap(scaledBitmap, targetWidth, targetHeight);
        } catch (Exception e) {
            Log.e(TAG, "Error loading and scaling resource: " + e.getMessage());
            return null;
        }
    }
    
    /**
     * Load and scale an image from a file
     * @param filePath Path to the image file
     * @param targetWidth Target width (use 0 for original size)
     * @param targetHeight Target height (use 0 for original size)
     * @return Scaled bitmap
     */
    public static Bitmap loadAndScaleFile(String filePath, int targetWidth, int targetHeight) {
        try {
            File file = new File(filePath);
            if (!file.exists()) {
                Log.e(TAG, "File does not exist: " + filePath);
                return null;
            }
            
            // If target dimensions are 0, use original size
            if (targetWidth == 0 || targetHeight == 0) {
                return BitmapFactory.decodeFile(filePath);
            }
            
            // First decode with inJustDecodeBounds=true to check dimensions
            final BitmapFactory.Options options = new BitmapFactory.Options();
            options.inJustDecodeBounds = true;
            BitmapFactory.decodeFile(filePath, options);
            
            // Calculate inSampleSize
            options.inSampleSize = calculateInSampleSize(options, targetWidth, targetHeight);
            
            // Decode bitmap with inSampleSize set
            options.inJustDecodeBounds = false;
            Bitmap scaledBitmap = BitmapFactory.decodeFile(filePath, options);
            
            return getResizedBitmap(scaledBitmap, targetWidth, targetHeight);
        } catch (Exception e) {
            Log.e(TAG, "Error loading and scaling file: " + e.getMessage());
            return null;
        }
    }
    
    /**
     * Load and scale an image from a stream
     * @param inputStream Input stream containing the image
     * @param targetWidth Target width (use 0 for original size)
     * @param targetHeight Target height (use 0 for original size)
     * @return Scaled bitmap
     */
    public static Bitmap loadAndScaleStream(InputStream inputStream, int targetWidth, int targetHeight) {
        try {
            // We need to reset the stream to read it twice
            if (!inputStream.markSupported()) {
                // If the stream doesn't support mark/reset, we can't use this method
                Log.e(TAG, "Input stream does not support mark/reset");
                return null;
            }
            
            // Mark the stream position
            inputStream.mark(inputStream.available());
            
            // If target dimensions are 0, use original size
            if (targetWidth == 0 || targetHeight == 0) {
                return BitmapFactory.decodeStream(inputStream);
            }
            
            // First decode with inJustDecodeBounds=true to check dimensions
            final BitmapFactory.Options options = new BitmapFactory.Options();
            options.inJustDecodeBounds = true;
            BitmapFactory.decodeStream(inputStream, null, options);
            
            // Reset the stream position
            inputStream.reset();
            
            // Calculate inSampleSize
            options.inSampleSize = calculateInSampleSize(options, targetWidth, targetHeight);
            
            // Decode bitmap with inSampleSize set
            options.inJustDecodeBounds = false;
            Bitmap scaledBitmap = BitmapFactory.decodeStream(inputStream, null, options);
            
            return getResizedBitmap(scaledBitmap, targetWidth, targetHeight);
        } catch (Exception e) {
            Log.e(TAG, "Error loading and scaling stream: " + e.getMessage());
            return null;
        }
    }
    
    /**
     * Load a image into an ImageView with auto-scaling
     * @param imageView Target ImageView
     * @param resourceId Resource ID of the image
     */
    public static void loadImageIntoView(ImageView imageView, int resourceId) {
        if (imageView == null) {
            Log.e(TAG, "ImageView is null");
            return;
        }
        
        Context context = imageView.getContext();
        
        // Get the dimensions of the ImageView
        int width = imageView.getWidth();
        int height = imageView.getHeight();
        
        // If the view dimensions are not available yet
        if (width <= 0 || height <= 0) {
            // Just load the image without scaling
            imageView.setImageResource(resourceId);
            return;
        }
        
        // Load and scale the image
        Bitmap scaledBitmap = loadAndScaleResource(context, resourceId, width, height);
        
        if (scaledBitmap != null) {
            imageView.setImageBitmap(scaledBitmap);
        } else {
            // Fallback to regular loading
            imageView.setImageResource(resourceId);
        }
    }
    
    /**
     * Load a image from a file into an ImageView with auto-scaling
     * @param imageView Target ImageView
     * @param filePath Path to the image file
     */
    public static void loadFileIntoView(ImageView imageView, String filePath) {
        if (imageView == null) {
            Log.e(TAG, "ImageView is null");
            return;
        }
        
        // Get the dimensions of the ImageView
        int width = imageView.getWidth();
        int height = imageView.getHeight();
        
        // If the view dimensions are not available yet
        if (width <= 0 || height <= 0) {
            // Post a runnable to try again after layout
            imageView.post(new Runnable() {
                @Override
                public void run() {
                    loadFileIntoView(imageView, filePath);
                }
            });
            return;
        }
        
        // Load and scale the image
        Bitmap scaledBitmap = loadAndScaleFile(filePath, width, height);
        
        if (scaledBitmap != null) {
            imageView.setImageBitmap(scaledBitmap);
        } else {
            Log.e(TAG, "Failed to load image from file: " + filePath);
        }
    }
    
    /**
     * Calculate the sample size for downsampling
     * @param options BitmapFactory.Options containing the image dimensions
     * @param reqWidth Requested width
     * @param reqHeight Requested height
     * @return Calculated sample size
     */
    private static int calculateInSampleSize(BitmapFactory.Options options, int reqWidth, int reqHeight) {
        // Raw height and width of image
        final int height = options.outHeight;
        final int width = options.outWidth;
        int inSampleSize = 1;
        
        if (height > reqHeight || width > reqWidth) {
            
            final int halfHeight = height / 2;
            final int halfWidth = width / 2;
            
            // Calculate the largest inSampleSize value that is a power of 2 and keeps both
            // height and width larger than the requested height and width.
            while ((halfHeight / inSampleSize) >= reqHeight
                    && (halfWidth / inSampleSize) >= reqWidth) {
                inSampleSize *= 2;
            }
        }
        
        return inSampleSize;
    }
    
    /**
     * Resize a bitmap to the target dimensions, maintaining aspect ratio
     * @param bitmap Original bitmap
     * @param targetWidth Target width
     * @param targetHeight Target height
     * @return Resized bitmap
     */
    private static Bitmap getResizedBitmap(Bitmap bitmap, int targetWidth, int targetHeight) {
        if (bitmap == null) {
            return null;
        }
        
        int width = bitmap.getWidth();
        int height = bitmap.getHeight();
        
        // No need to resize
        if (width == targetWidth && height == targetHeight) {
            return bitmap;
        }
        
        // Calculate the scale factor to maintain aspect ratio
        float scaleWidth = ((float) targetWidth) / width;
        float scaleHeight = ((float) targetHeight) / height;
        
        // Use the smaller scale factor to ensure the image fits within the target dimensions
        float scaleFactor = Math.min(scaleWidth, scaleHeight);
        
        // Create a matrix for the scaling operation
        Matrix matrix = new Matrix();
        matrix.postScale(scaleFactor, scaleFactor);
        
        // Create the new bitmap
        Bitmap resizedBitmap = Bitmap.createBitmap(
                bitmap, 0, 0, width, height, matrix, true);
        
        // Recycle the original bitmap if it's not being used elsewhere
        if (resizedBitmap != bitmap) {
            bitmap.recycle();
        }
        
        return resizedBitmap;
    }
    
    /**
     * Convert a drawable to a bitmap
     * @param drawable Source drawable
     * @return Bitmap representation of the drawable
     */
    public static Bitmap drawableToBitmap(Drawable drawable) {
        if (drawable instanceof BitmapDrawable) {
            return ((BitmapDrawable) drawable).getBitmap();
        }
        
        int width = drawable.getIntrinsicWidth();
        int height = drawable.getIntrinsicHeight();
        
        // Handle null size drawables
        if (width <= 0) width = 1;
        if (height <= 0) height = 1;
        
        Bitmap bitmap = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888);
        android.graphics.Canvas canvas = new android.graphics.Canvas(bitmap);
        drawable.setBounds(0, 0, canvas.getWidth(), canvas.getHeight());
        drawable.draw(canvas);
        
        return bitmap;
    }
}