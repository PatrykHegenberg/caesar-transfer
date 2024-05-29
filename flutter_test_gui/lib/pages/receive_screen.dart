import 'dart:io';

import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test_gui/main.dart';
import 'package:flutter_test_gui/pages/transfer_screen.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:permission_handler/permission_handler.dart';
import 'package:shared_preferences/shared_preferences.dart';
// import 'package:flutter_test_gui/src/rust/api/simple.dart';
// import 'package:flutter_test_gui/src/rust/frb_generated.dart';
import 'package:flutter_test_gui/consts/consts.dart';

/// Screen for receiving files.
///
/// This screen is used to accept incoming file transfers. It displays a QR
/// code scanner on supported platforms and allows the user to enter a
/// connection link manually.
class ReceiveScreen extends StatefulWidget {
  /// Creates a new instance of the receive screen.
  const ReceiveScreen({super.key});

  @override
  ReceiveScreenState createState() => ReceiveScreenState();
}

/// State for the receive screen.
class ReceiveScreenState extends State<ReceiveScreen> {
  /// The URL of the app that initiated the transfer.
  String appOrigin = '';

  /// Text editing controller for the connection link input.
  final myController = TextEditingController();

  /// The current input value of the connection link input.
  String inputValue = '';

  /// Whether to show the QR code scanner.
  bool _showScanner = false;

  /// Builds the QR code scanner widget.
  ///
  /// If the platform is iOS or Android, a QR code scanner is displayed. If the
  /// platform is not supported, an empty box is returned.
  ///
  /// Returns a QR code scanner widget if the platform is supported, otherwise an
  /// empty box.
  Widget _buildQRScanner() {
    // Check if the platform is iOS or Android
    if (Platform.isIOS || Platform.isAndroid) {
      return MobileScanner(
        controller: MobileScannerController(
            detectionSpeed: DetectionSpeed.noDuplicates),
        onDetect: (barcode) {
          // Check if the scanner failed to scan a QR code
          if (barcode.raw == null) {
            debugPrint('Failed to scan qr code');
          } else {
            // Set the input value to the scanned code
            final String code = barcode.barcodes.first.displayValue.toString();
            print(code);
            setState(() {
              inputValue = code;
              _showScanner = false;
            });
          }
        },
      );
    } else {
      // If the platform is not supported, hide the scanner
      _showScanner = false;
      return const SizedBox.shrink();
    }
  }

  /// Loads the app origin from the shared preferences.
  ///
  /// If the app origin is not present in the shared preferences, it sets the
  /// default value to 'wss://caesar-transfer-iu.shuttleapp.rs'.
  ///
  /// Returns a [Future] that completes with no value.
  Future<void> loadSettings() async {
    SharedPreferences prefs = await SharedPreferences.getInstance();
    appOrigin = prefs.getString('app_origin') ??
        'wss://caesar-transfer-iu.shuttleapp.rs'; // Load the app origin
  }

  /// Requests permission for a given [permission].
  ///
  /// If the permission is already granted, it returns true. If the permission
  /// is not granted, it requests the permission and returns true if the user
  /// grants the permission, otherwise it returns false.
  ///
  /// Returns a [Future] that completes with a [bool] value indicating whether
  /// the permission was granted or not.
  Future<bool> _requestPermission(Permission permission) async {
    // Print the function name
    print("In _requestPermission");

    // Check if the permission is already granted
    if (await permission.isGranted) {
      // Print the message
      print("Granted");
      return true;
    } else {
      // Print the message
      print("Else Zweig");

      // Request the permission
      final result = await permission.request();

      // Check if the permission is granted
      if (result == PermissionStatus.granted) {
        return true;
      } else {
        return false;
      }
    }
  }

  /// Starts a transfer by getting the directory path from the user and navigating
  /// to the [TransferScreen] with the given [input] and [filePath].
  ///
  /// The [input] is the value of the input field. If it is not empty, it gets the
  /// directory path from the user using the [FilePicker.platform.getDirectoryPath]
  /// method. If the user chooses a directory, it sets the [filePath] to the
  /// selected directory path. If the user doesn't choose a directory, it prints
  /// a message indicating that the user didn't choose a directory.
  ///
  /// If the platform is Android, it checks if the external storage permission is
  /// granted. If it is not granted, it requests the permission and navigates to
  /// the [TransferScreen] with the given [input] and [filePath]. If the permission
  /// is granted, it navigates to the [MyHomePage] with the title 'Caesar Transfer'.
  ///
  /// If the platform is not Android, it navigates to the [TransferScreen] with the
  /// given [input] and [filePath].
  ///
  /// Returns a [Future] that completes with no value.
  Future<void> _startTransfer(String appOrigin) async {
    final input = inputValue.trim();
    String filePath = '';
    if (input.isNotEmpty) {
      // Get the directory path from the user
      String? selectDirectory = await FilePicker.platform.getDirectoryPath();
      if (selectDirectory == null) {
        // Print a message indicating that the user didn't choose a directory
        print("User doesn't choose a directory");
      } else {
        // Set the filePath to the selected directory path
        print("User chose: $selectDirectory");
        filePath = selectDirectory;
      }
      if (Platform.isAndroid) {
        // Check if the external storage permission is granted
        if (await _requestPermission(Permission.manageExternalStorage)) {
          // Navigate to the TransferScreen
          Navigator.push(
              context,
              MaterialPageRoute(
                  builder: (context) => TransferScreen(
                      transferName: input, directory: filePath)));
        } else {
          // Navigate to the MyHomePage with the title 'Caesar Transfer'
          Navigator.push(
              context,
              MaterialPageRoute(
                  builder: (context) =>
                      const MyHomePage(title: 'Caesar Transfer')));
        }
      } else {
        // Navigate to the TransferScreen
        Navigator.push(
            context,
            MaterialPageRoute(
                builder: (context) =>
                    TransferScreen(transferName: input, directory: filePath)));
      }
    }
  }

  /// Builds the scaffold for the receive screen.
  ///
  /// The scaffold contains a center widget that contains a column of widgets.
  /// The column contains a QR code scanner if [_showScanner] is true, otherwise
  /// it contains a text field for entering the transfer name. Below the text
  /// field is an elevated button for initiating the receive process.
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Constants.backColor,
      body: Center(
          child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          // If _showScanner is false, display a QR code icon that can be tapped
          // to start the QR code scanner.
          if (!_showScanner)
            GestureDetector(
              onTap: () {
                if (Platform.isIOS || Platform.isAndroid) {
                  setState(() {
                    _showScanner = true;
                  });
                }
              },
              child: Container(
                width: 200,
                height: 200,
                decoration: const BoxDecoration(
                  shape: BoxShape.circle,
                  color: Constants.textColor,
                ),
                child: const Center(
                  child: Icon(
                    Icons.qr_code,
                    color: Constants.highlightColor,
                    size: 100,
                  ),
                ),
              ),
            ),
          // If _showScanner is true, display the QR code scanner.
          if (_showScanner)
            Container(
              width: MediaQuery.of(context).size.width * 0.8,
              height: MediaQuery.of(context).size.height * 0.5,
              decoration: BoxDecoration(
                color: Colors.white,
                borderRadius: BorderRadius.circular(16),
              ),
              child: _buildQRScanner(),
            ),
          // Add some spacing between the scanner and the text field.
          const SizedBox(height: 32),
          // Display a text field for entering the transfer name.
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: SizedBox(
              width: MediaQuery.of(context).size.width * 0.5,
              child: TextField(
                controller: myController,
                textAlign: TextAlign.center,
                style: const TextStyle(
                  color: Constants.highlightColor,
                ),
                onChanged: (value) {
                  setState(() {
                    inputValue = value;
                  });
                },
                decoration: const InputDecoration(
                  labelText: 'Enter Transfername',
                  alignLabelWithHint: true,
                  floatingLabelAlignment: FloatingLabelAlignment.center,
                  labelStyle: TextStyle(color: Constants.textColor),
                  enabledBorder: UnderlineInputBorder(
                    borderSide: BorderSide(color: Constants.textColor),
                  ),
                  focusedBorder: UnderlineInputBorder(
                    borderSide: BorderSide(color: Constants.textColor),
                  ),
                ),
              ),
            ),
          ),
          // Add some spacing between the text field and the receive button.
          const SizedBox(height: 16),
          // Display an elevated button for initiating the receive process.
          ElevatedButton(
            style: ElevatedButton.styleFrom(
              backgroundColor: Constants.textColor,
              foregroundColor: Constants.backColor,
              shape: RoundedRectangleBorder(
                borderRadius: BorderRadius.circular(20),
              ),
            ),
            onPressed: () {
              loadSettings().then((_) => _startTransfer(appOrigin));
            },
            child: const Text('Receive'),
          ),
        ],
      )),
    );
  }
}
