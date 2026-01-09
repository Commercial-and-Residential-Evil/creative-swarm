package commercial_and_residential_evil.whirled_peas

import com.google.androidgamesdk.GameActivity

/**
 * Main activity for Whirled Peas Visualiser.
 *
 * This activity uses Google's GameActivity which provides the native window
 * surface that Bevy/wgpu needs to render to. The native library is loaded
 * automatically based on the android.app.lib_name metadata in the manifest.
 */
class MainActivity : GameActivity() {
    companion object {
        init {
            // Load the native library
            System.loadLibrary("whirled_peas")
        }
    }
}
