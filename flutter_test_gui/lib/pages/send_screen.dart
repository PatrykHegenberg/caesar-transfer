import 'dart:async';

import 'package:flutter/material.dart';
import 'package:desktop_drop/desktop_drop.dart';
import 'package:cross_file/cross_file.dart';
import 'package:file_picker/file_picker.dart';
import 'package:flutter_test_gui/pages/waiting_screen.dart';
import 'package:flutter_test_gui/src/rust/api/simple.dart';
// import 'package:flutter_test_gui/src/rust/frb_generated.dart';
import 'package:flutter_test_gui/consts/consts.dart';

/// Represents the screen for sending files.
///
/// This is a [StatefulWidget] that displays a screen for sending files.
/// It allows the user to select files to send and provides a name for the transfer.
/// The selected files are stored in a list and can be accessed by the [SendScreenState].
///
/// See also:
///   - [SendScreenState]
class SendScreen extends StatefulWidget {
  /// Creates a [SendScreen].
  ///
  /// The [key] parameter is used to identify the [SendScreen] widget.
  const SendScreen({super.key});

  /// Creates a [SendScreenState] to control the [SendScreen].
  ///
  /// This method is called when a [SendScreen] widget is created.
  /// It returns a new instance of [SendScreenState].
  @override
  SendScreenState createState() => SendScreenState();
}

class SendScreenState extends State<SendScreen> {
  /// List of selected files to send.
  final List<XFile> _list = [];

  /// Name of the transfer.
  String transferName = '';

  /// Indicates whether the user is currently dragging files.
  bool _dragging = false;

  /// Opens the file picker and adds the selected files to [_list].
  ///
  /// See also:
  ///   - [FilePicker.platform.pickFiles]
  Future<void> openFilePicker() async {
    FilePickerResult? result = await FilePicker.platform.pickFiles(
      allowMultiple: true, // Allow selecting multiple files
    );

    if (result != null) {
      _list.addAll(result.xFiles);
    }
  }

  /// Generates a random name for the transfer and navigates to the waiting screen.
  ///
  /// See also:
  ///   - [generateRandomName]
  ///   - [WaitingScreen]
  Future<void> _startTransfer() async {
    final randomName =
        generateRandomName(); // Call Rust function to generate random name
    print('Generated transfer name: $randomName');
    setState(() {
      transferName = randomName;
    });
    Navigator.push(
        context,
        MaterialPageRoute(
            builder: (context) =>
                WaitingScreen(transferName: transferName, files: _list)));
  }

  /// Builds the UI for the send screen.
  ///
  /// Returns a [Scaffold] widget that contains a [Column] with two children:
  /// - A [Center] widget that contains a [Stack] with a [GestureDetector] that
  ///   handles file picking and dragging.
  /// - An [ElevatedButton] that triggers the transfer when pressed.
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      // Set the background color of the scaffold.
      backgroundColor: Constants.backColor,
      // Build the body of the scaffold.
      body: Column(
        // Align the children vertically to the center.
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          // Build the file picking and dragging UI.
          Center(
            child: Stack(
              children: [
                // Build the gesture detector.
                GestureDetector(
                  // Handle file picking when the user taps.
                  onTap: openFilePicker,
                  // Handle file dragging.
                  child: DropTarget(
                    // Add the selected files to the list when the user drops files.
                    onDragDone: (detail) {
                      setState(() {
                        _list.addAll(detail.files);
                      });
                    },
                    // Show the add icon when the user drags files over the drop area.
                    onDragEntered: (detail) {
                      setState(() {
                        _dragging = true;
                      });
                    },
                    // Hide the add icon when the user stops dragging files.
                    onDragExited: (detail) {
                      setState(() {
                        _dragging = false;
                      });
                    },
                    // Build the drop area UI.
                    child: Column(
                      children: [
                        // Build the circular container for the drop area.
                        Container(
                          height: 200,
                          width: 200,
                          decoration: const BoxDecoration(
                              shape: BoxShape.circle,
                              color: Constants.textColor),
                          // Show the add icon when the user is dragging files.
                          child: _dragging
                              ? const Center(
                                  child: Icon(
                                    Icons.add_rounded,
                                    color: Constants.highlightColor,
                                    size: 200,
                                  ),
                                )
                              // Show the upload icon when the user is not dragging files.
                              : const Center(
                                  child: Icon(
                                    Icons.upload_rounded,
                                    color: Constants.highlightColor,
                                    size: 200,
                                  ),
                                ),
                        ),
                        // Add some spacing between the drop area and the send button.
                        const SizedBox(height: 16),
                      ],
                    ),
                  ),
                ),
              ],
            ),
          ),
          // Build the send button.
          ElevatedButton(
            style: ElevatedButton.styleFrom(
              // Set the background color of the button.
              backgroundColor: Constants.textColor,
              // Set the text color of the button.
              foregroundColor: Constants.backColor,
              // Set the shape of the button.
              shape: RoundedRectangleBorder(
                borderRadius: BorderRadius.circular(20),
              ),
            ),
            // Trigger the transfer when the user presses the button.
            onPressed: () {
              _startTransfer();
            },
            // Set the text of the button.
            child: const Text("Send"),
          ),
        ],
      ),
    );
  }
}
