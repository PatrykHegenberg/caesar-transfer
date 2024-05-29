import 'package:flutter/material.dart';
// import 'package:flutter_test_gui/pages/send_screen.dart';
import 'package:flutter_test_gui/main.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:cross_file/cross_file.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:flutter_test_gui/src/rust/api/simple.dart';
// import 'package:flutter_test_gui/src/rust/frb_generated.dart';
import 'package:flutter_test_gui/consts/consts.dart';

/// Represents the screen for displaying the waiting state.
///
/// This is a [StatefulWidget] that displays a screen for the waiting state.
/// It takes in two parameters:
///   - [transferName]: The name of the transfer.
///   - [files]: The list of files being transferred.
class WaitingScreen extends StatefulWidget {
  // The list of files being transferred.
  final List<XFile> files;

  // The name of the transfer.
  final String transferName;

  /// Creates a new instance of the [WaitingScreen] widget.
  ///
  /// The [transferName] parameter is the name of the transfer.
  /// The [files] parameter is the list of files being transferred.
  const WaitingScreen(
      {Key? key, required this.transferName, required this.files})
      : super(key: key);

  /// Creates the mutable state for this widget at a given location in the tree.
  ///
  /// See also:
  ///   - [StatefulWidget.createState]
  @override
  WaitingScreenState createState() => WaitingScreenState();
}

class WaitingScreenState extends State<WaitingScreen> {
  // The origin of the app.
  String appOrigin = '';

  /// Initializes the state of the widget.
  ///
  /// This function is called when the widget is first created.
  @override
  void initState() {
    super.initState();

    // Load the settings and then start the transfer.
    loadSettings().then((_) => callStartSender(appOrigin));
  }

  /// Loads the settings.
  ///
  /// This function loads the settings from the shared preferences.
  /// It retrieves the app origin from the shared preferences and assigns it to
  /// the [appOrigin] variable.
  ///
  /// Returns a [Future] that completes when the settings are loaded.
  Future<void> loadSettings() async {
    // Get the shared preferences instance.
    SharedPreferences prefs = await SharedPreferences.getInstance();

    // Get the app origin from the shared preferences.
    // If the app origin is not found, use the default value.
    appOrigin = prefs.getString('app_origin') ??
        'wss://caesar-transfer-iu.shuttleapp.rs';
  }

  /// Calls the start sender function.
  ///
  /// This function calls the [_startTransfer] function with the provided
  /// [appOrigin].
  ///
  /// Parameters:
  ///   - appOrigin: The origin of the app.
  Future<void> callStartSender(String appOrigin) async {
    _startTransfer(appOrigin);
  }

  /// Starts the transfer.
  ///
  /// This function converts the list of files to a list of file names and then
  /// calls the [startRustSender] function with the provided parameters.
  ///
  /// Parameters:
  ///   - appOrigin: The origin of the app.
  Future<void> _startTransfer(String appOrigin) async {
    // Convert the list of files to a list of file names.
    List<String> fileNames = widget.files.map((file) => file.path).toList();

    // Start the transfer.
    final outcome = await startRustSender(
        name: widget.transferName, relay: appOrigin, files: fileNames);

    // Navigate to the home page.
    Navigator.push(
        context,
        MaterialPageRoute(
            builder: (context) => MyHomePage(title: 'Caesar Transfer')));
  }

  /// Builds the waiting screen widget.
  ///
  /// This widget displays the transfer name and a QR code representing the
  /// transfer name.
  ///
  /// Returns:
  ///   A [Scaffold] widget containing the waiting screen UI.
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      // Set the background color of the scaffold.
      backgroundColor: Constants.backColor,
      // Center the content of the scaffold.
      body: Center(
        child: Column(
          // Align the children of the column in the center.
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            // Display the transfer name.
            Text(
              widget.transferName,
              // Set the text style for the transfer name.
              style: const TextStyle(color: Colors.white, fontSize: 24),
            ),
            // Add spacing between the transfer name and the QR code.
            const SizedBox(height: 32),
            // Display a QR code representing the transfer name.
            QrImageView(
              // Set the data to be encoded in the QR code.
              data: widget.transferName,
              // Set the version of the QR code.
              version: QrVersions.auto,
              // Set the size of the QR code.
              size: 200,
              // Set the foreground color of the QR code.
              foregroundColor: Constants.highlightColor,
            ),
          ],
        ),
      ),
    );
  }
}
