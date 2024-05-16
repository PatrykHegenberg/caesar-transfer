import 'dart:async';

import 'package:flutter/material.dart';
import 'package:desktop_drop/desktop_drop.dart';
import 'package:cross_file/cross_file.dart';
import 'package:file_picker/file_picker.dart';
import 'package:flutter_test_gui/pages/waiting_screen.dart';
import 'package:flutter_test_gui/src/rust/api/simple.dart';
import 'package:flutter_test_gui/src/rust/frb_generated.dart';

const backColor = Color(0xFF32363E);
const highlightColor = Color(0xFF98C379);
const textColor = Color(0xFFABB2BF);

class SendScreen extends StatefulWidget {
  @override
  _SendScreenState createState() => _SendScreenState();
}

class _SendScreenState extends State<SendScreen> {
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
      backgroundColor: backColor,
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
                          decoration: BoxDecoration(
                              shape: BoxShape.circle,
                              color: _dragging
                                  ? Colors.blue.withOpacity(0.4)
                                  : textColor),
                          child: _list.isEmpty
                              ? const Center(
                                  child: Icon(
                                    Icons.add_circle_outlined,
                                    color: highlightColor,
                                    size: 200,
                                  ),
                                )
                              : Text(_list.join("\n")),
                        ),
                        const SizedBox(height: 16),
                        if (_list.isNotEmpty)
                          SizedBox(
                            height: 100,
                            child: ListView.builder(
                              itemCount: _list.length,
                              itemBuilder: (context, index) {
                                return Text(
                                  _list[index].name,
                                  style: const TextStyle(color: Colors.white),
                                );
                              },
                            ),
                          )
                      ],
                    ),
                  ),
                ),
              ],
            ),
          ),
          ElevatedButton(
            style: ElevatedButton.styleFrom(
              backgroundColor: textColor,
              foregroundColor: highlightColor,
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
