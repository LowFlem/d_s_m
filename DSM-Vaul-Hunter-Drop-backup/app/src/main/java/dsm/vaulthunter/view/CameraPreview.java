package dsm.vaulthunter.view;

import android.content.Context;
import android.hardware.Camera;
import android.util.Log;
import android.view.SurfaceHolder;
import android.view.SurfaceView;

import java.io.IOException;

/**
 * A basic Camera preview class for legacy devices
 * Note: This is used as a fallback for devices that don't support CameraX
 */
public class CameraPreview extends SurfaceView implements SurfaceHolder.Callback {
    private static final String TAG = "CameraPreview";
    
    private final SurfaceHolder holder;
    private Camera camera;
    
    /**
     * Constructor
     * @param context Application context
     * @param camera Camera instance
     */
    @SuppressWarnings("deprecation") // Using Camera API for backwards compatibility
    public CameraPreview(Context context, Camera camera) {
        super(context);
        this.camera = camera;
        
        // Install a SurfaceHolder.Callback
        holder = getHolder();
        holder.addCallback(this);
        
        // Deprecated but required on Android versions before 3.0
        holder.setType(SurfaceHolder.SURFACE_TYPE_PUSH_BUFFERS);
    }
    
    /**
     * Set a new camera instance
     * @param camera Camera instance
     */
    public void setCamera(Camera camera) {
        this.camera = camera;
        if (holder.getSurface() != null && camera != null) {
            try {
                camera.setPreviewDisplay(holder);
                camera.startPreview();
            } catch (IOException e) {
                Log.e(TAG, "Error setting camera preview: " + e.getMessage());
            }
        }
    }
    
    @Override
    public void surfaceCreated(SurfaceHolder holder) {
        // Surface created, now we can set up the camera preview
        try {
            if (camera != null) {
                camera.setPreviewDisplay(holder);
                camera.startPreview();
            }
        } catch (IOException e) {
            Log.e(TAG, "Error setting camera preview: " + e.getMessage());
        }
    }
    
    @Override
    public void surfaceChanged(SurfaceHolder holder, int format, int width, int height) {
        // If the surface changed, we need to reset the preview
        if (this.holder.getSurface() == null || camera == null) {
            return;
        }
        
        try {
            // Stop preview before making changes
            camera.stopPreview();
            
            // Set the best preview size
            Camera.Parameters parameters = camera.getParameters();
            Camera.Size bestSize = getBestPreviewSize(width, height, parameters);
            if (bestSize != null) {
                parameters.setPreviewSize(bestSize.width, bestSize.height);
                camera.setParameters(parameters);
            }
            
            // Start preview with new settings
            camera.setPreviewDisplay(this.holder);
            camera.startPreview();
        } catch (Exception e) {
            Log.e(TAG, "Error restarting camera preview: " + e.getMessage());
        }
    }
    
    @Override
    public void surfaceDestroyed(SurfaceHolder holder) {
        // Surface is destroyed, do not release the camera here - this should be done in the activity
    }
    
    /**
     * Find the best preview size for the camera
     * @param targetWidth Target width
     * @param targetHeight Target height
     * @param parameters Camera parameters
     * @return Best preview size
     */
    private Camera.Size getBestPreviewSize(int targetWidth, int targetHeight, Camera.Parameters parameters) {
        Camera.Size bestSize = null;
        int bestArea = 0;
        float targetRatio = (float) targetWidth / targetHeight;
        
        // Find the largest size with the closest aspect ratio
        for (Camera.Size size : parameters.getSupportedPreviewSizes()) {
            int area = size.width * size.height;
            float ratio = (float) size.width / size.height;
            float aspectDiff = Math.abs(ratio - targetRatio);
            
            // Consider both size and aspect ratio
            if (area > bestArea && aspectDiff < 0.2) {
                bestArea = area;
                bestSize = size;
            }
        }
        
        return bestSize;
    }
}