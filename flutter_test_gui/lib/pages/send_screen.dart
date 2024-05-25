import 'dart:async';

import 'package:flutter/material.dart';
import 'package:desktop_drop/desktop_drop.dart';
import 'package:cross_file/cross_file.dart';
import 'package:file_picker/file_picker.dart';
import 'package:flutter_test_gui/pages/waiting_screen.dart';
import 'package:flutter_test_gui/src/rust/api/simple.dart';
// import 'package:flutter_test_gui/src/rust/frb_generated.dart';
import 'package:flutter_test_gui/consts/consts.dart';

class SendScreen extends StatefulWidget {
  const SendScreen({super.key});

  @override
  SendScreenState createState() => SendScreenState();
}

class SendScreenState extends State<SendScreen> {
  final List<XFile> _list = [];
  String transferName = '';
  bool _dragging = false;
  Future<void> openFilePicker() async {
    FilePickerResult? result = await FilePicker.platform.pickFiles(
      allowMultiple: true, // Erlaube die Auswahl mehrerer Dateien
    );

    if (result != null) {
      _list.addAll(result.xFiles);
    }
  }

  Future<void> _startTransfer() async {
    final randomName = generateRandomName(); // Rust-Funktion aufrufen
    print('Zusammengefügter Text: $randomName');
    setState(() {
      transferName = randomName;
    });
    Navigator.push(
        context,
        MaterialPageRoute(
            builder: (context) =>
                WaitingScreen(transferName: transferName, files: _list)));
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Constants.backColor,
      body: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Center(
            child: Stack(
              children: [
                GestureDetector(
                  onTap: openFilePicker,
                  child: DropTarget(
                    onDragDone: (detail) {
                      setState(() {
                        _list.addAll(detail.files);
                      });
                    },
                    onDragEntered: (detail) {
                      setState(() {
                        _dragging = true;
                      });
                    },
                    onDragExited: (detail) {
                      setState(() {
                        _dragging = false;
                      });
                    },
                    child: Column(
                      children: [
                        Container(
                          height: 200,
                          width: 200,
                          decoration: const BoxDecoration(
                              shape: BoxShape.circle,
                              color: Constants.textColor),
                          child: _dragging
                              ? const Center(
                                  child: Icon(
                                    Icons.add_rounded,
                                    color: Constants.highlightColor,
                                    size: 200,
                                  ),
                                )
                              : const Center(
                                  child: Icon(
                                    Icons.upload_rounded,
                                    color: Constants.highlightColor,
                                    size: 200,
                                  ),
                                ),
                        ),
                        const SizedBox(height: 16),
                      ],
                    ),
                  ),
                ),
              ],
            ),
          ),
          ElevatedButton(
            style: ElevatedButton.styleFrom(
              backgroundColor: Constants.textColor,
              foregroundColor: Constants.highlightColor,
              shape: RoundedRectangleBorder(
                borderRadius: BorderRadius.circular(20),
              ),
            ),
            onPressed: () {
              _startTransfer();
            },
            child: const Text("Send"),
          ),
        ],
      ),
    );
  }
}
