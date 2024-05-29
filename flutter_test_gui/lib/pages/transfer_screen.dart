import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test_gui/main.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:flutter_test_gui/src/rust/api/simple.dart';
// import 'package:flutter_test_gui/src/rust/frb_generated.dart';
import 'package:flutter_test_gui/consts/consts.dart';
import 'package:permission_handler/permission_handler.dart';

/// Represents the screen for transferring files.
///
/// This is a [StatefulWidget] that displays a screen for transferring files.
/// It takes in two parameters:
///   - [transferName]: The name of the transfer.
///   - [directory]: The directory containing the files to be transferred.
class TransferScreen extends StatefulWidget {
  // The name of the transfer.
  final String transferName;

  // The directory containing the files to be transferred.
  final String directory;

  /// Creates a [TransferScreen] widget.
  ///
  /// The [transferName] and [directory] parameters are required.
  ///
  /// The [key] parameter is optional.
  const TransferScreen(
      {Key? key, required this.transferName, required this.directory})
      : super(key: key);

  @override
  TransferScreenState createState() => TransferScreenState();
}

class TransferScreenState extends State<TransferScreen> {
  // The origin of the application.
  String appOrigin = '';

  // The input value of the transfer name.
  String inputValue = '';

  @override
  void initState() {
    // Call the loadSettings function to load the settings.
    super.initState();
    loadSettings().then((_) => callStartReceiver(appOrigin));
  }

  /// Loads the settings from the SharedPreferences.
  ///
  /// It retrieves the value of 'app_origin' from the SharedPreferences and
  /// assigns it to the [appOrigin] variable. If the value is not present, it
  /// assigns a default value of 'wss://caesar-transfer-iu.shuttleapp.rs'.
  Future<void> loadSettings() async {
    SharedPreferences prefs = await SharedPreferences.getInstance();
    appOrigin = prefs.getString('app_origin') ??
        'wss://caesar-transfer-iu.shuttleapp.rs';
  }

  /// Calls the start transfer function with the given [appOrigin].
  ///
  /// It calls the _startTransfer function with the [appOrigin] parameter.
  Future<void> callStartReceiver(String appOrigin) async {
    _startTransfer(appOrigin);
  }

  /// Starts the transfer with the given [appOrigin].
  ///
  /// If the transfer name is not empty, it checks if the platform is Android.
  /// If it is, it requests the ManageExternalStorage permission. If the
  /// permission is granted, it starts the receiver using the startRustReceiver
  /// function. If the permission is not granted, it navigates to the
  /// MyHomePage. If the platform is not Android, it starts the receiver
  /// directly. If the transfer name is empty, it does not start the receiver.
  ///
  /// Parameters:
  ///   - appOrigin: The origin of the application.
  Future<void> _startTransfer(String appOrigin) async {
    // Get the input value from the widget.
    final input = widget.transferName;
    String filePath = widget.directory;

    // If the input value is not empty, start the transfer.
    if (input.isNotEmpty) {
      if (Platform.isAndroid) {
        // Check if the ManageExternalStorage permission is granted.
        //if (await _requestPermission(Permission.manageExternalStorage)) {
        try {
          // Start the receiver with the given parameters.
          final outcome = await startRustReceiver(
              filepath: filePath, transfername: input, relay: appOrigin);
          print('Ergebnis von Rust: $outcome');
        } catch (e) {
          // If an error occurs, print the error message.
          print('Fehler beim Starten des Receivers: $e');
        }
        // Navigate to the MyHomePage.
        Navigator.push(
            context,
            MaterialPageRoute(
                builder: (context) =>
                    const MyHomePage(title: 'Caesar Transfer')));
        //} else {
        //  // If the permission is not granted, navigate to the MyHomePage.
        //  Navigator.push(
        //      context,
        //      MaterialPageRoute(
        //          builder: (context) =>
        //              const MyHomePage(title: 'Caesar Transfer')));
        //}
      } else {
        // If the platform is not Android, start the receiver directly.
        try {
          final outcome = await startRustReceiver(
              filepath: filePath, transfername: input, relay: appOrigin);
          print('Ergebnis von Rust: $outcome');
        } catch (e) {
          // If an error occurs, print the error message.
          print('Fehler beim Starten des Receivers: $e');
        }
        // Navigate to the MyHomePage.
        Navigator.push(
            context,
            MaterialPageRoute(
                builder: (context) =>
                    const MyHomePage(title: 'Caesar Transfer')));
      }
    }
    // Print the app origin.
    print("Transfer startet with app_origin: $appOrigin");
  }

  /// Requests the given [permission] and returns a `Future` of `bool` indicating
  /// whether the permission is granted.
  ///
  /// If the permission is already granted, it returns `true`. If the permission
  /// is not granted, it requests the permission and returns `true` if the
  /// permission is granted successfully, otherwise it returns `false`.
  ///
  /// Parameters:
  ///   - permission: The permission to be requested.
  ///
  /// Returns:
  ///   A `Future` of `bool` indicating whether the permission is granted.
  Future<bool> _requestPermission(Permission permission) async {
    // If the permission is already granted, return true.
    if (await permission.isGranted) {
      return true;
    } else {
      // Request the permission and get the result.
      var result = await permission.request();
      // If the permission is granted, return true. Otherwise, return false.
      return result == PermissionStatus.granted;
    }
  }

  @override

  /// Builds the widget tree for the TransferScreen.
  ///
  /// This method builds a widget tree for the TransferScreen. It returns a
  /// Scaffold widget with a background color set to Constants.backColor. The
  /// body of the scaffold is a Center widget that contains a Column widget.
  /// The Column widget has its mainAxisAlignment set to MainAxisAlignment.center.
  /// It contains three children: a Text widget displaying the transferName, a
  /// Text widget with the text "Transfer in Progress", and a SizedBox widget
  /// with a height of 32. The SizedBox widget is followed by a Center widget
  /// that contains an Icon widget with the icon Icons.cloud_download_rounded,
  /// its color set to Constants.highlightColor, and its size set to 200.
  ///
  /// Returns:
  ///   The widget tree for the TransferScreen.
  Widget build(BuildContext context) {
    return Scaffold(
      // Set the background color of the Scaffold widget.
      backgroundColor: Constants.backColor,
      body: Center(
        // The body of the Scaffold widget.
        child: Column(
          // The Column widget has its mainAxisAlignment set to
          // MainAxisAlignment.center.
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            // A Text widget displaying the transferName.
            Text(
              widget.transferName,
              style: const TextStyle(
                color: Colors.white,
                fontSize: 24,
              ),
            ),
            // A Text widget with the text "Transfer in Progress".
            Text("Transfer in Progress"),
            // A SizedBox widget with a height of 32.
            const SizedBox(height: 32),
            // A Center widget containing an Icon widget.
            const Center(
              child: Icon(
                Icons.cloud_download_rounded,
                color: Constants.highlightColor,
                size: 200,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
